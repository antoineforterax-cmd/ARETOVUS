#!/usr/bin/env python3
"""Run a two-process ARETOVUS testnet propagation and convergence smoke test."""
from __future__ import annotations

import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARGO = shutil.which("cargo") or str(Path.home() / ".cargo" / "bin" / "cargo")
RECIPIENT = "43" * 32


def run_command(binary: Path, *args: str) -> str:
    result = subprocess.run(
        [str(binary), *args], cwd=ROOT, text=True, capture_output=True, timeout=120
    )
    output = result.stdout + result.stderr
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {output}")
    return output


def run_node(binary: Path, *args: str) -> str:
    return run_command(binary, *args)


def run_wallet(binary: Path, *args: str) -> str:
    return run_command(binary, *args)





def free_address() -> tuple[str, int]:
    with socket.socket() as candidate:
        candidate.bind(("127.0.0.1", 0))
        return ("127.0.0.1", candidate.getsockname()[1])


def wait_listening(process: subprocess.Popen[str], address: tuple[str, int], log: Path) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"testnet server exited early; log: {log.read_text(errors='replace')}")
        try:
            with socket.create_connection(address, timeout=0.25):
                return
        except OSError:
            time.sleep(0.1)
    raise TimeoutError(f"testnet server did not listen at {address}; log: {log.read_text(errors='replace')}")


def crash_recovery_test(binary: Path, data_dir: Path, miner: str) -> int:
    process = subprocess.Popen(
        [str(binary), "testnet", "mine", "--data-dir", str(data_dir),
         "--miner", miner, "--count", "10000"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    deadline = time.monotonic() + 30
    saved_height = -1
    saved_tip = None
    try:
        while time.monotonic() < deadline:
            # Capture the immutable manifest identity *before* killing the
            # miner. Looking at file names after the kill can miss the very
            # snapshot we intended to protect from accidental loss.
            snapshots = [
                (int(match.group(1)), match.group(2))
                for snapshot in data_dir.glob("snapshot-*.aretidx")
                if (match := re.fullmatch(
                    r"snapshot-(\d{20})-([0-9a-f]{64})-[0-9a-f]{64}\.aretidx",
                    snapshot.name,
                ))
            ]
            if snapshots and max(height for height, _ in snapshots) >= 2:
                saved_height = max(height for height, _ in snapshots)
                saved_tips = {
                    tip for height, tip in snapshots if height == saved_height
                }
                if len(saved_tips) != 1:
                    raise AssertionError(
                        f"ambiguous snapshot tip at height {saved_height}: "
                        f"{sorted(saved_tips)}"
                    )
                saved_tip = next(iter(saved_tips))
                process.kill()
                process.wait(timeout=5)
                break
            if process.poll() is not None:
                raise RuntimeError("miner exited before the forced-stop checkpoint")
            time.sleep(0.005)
        else:
            raise TimeoutError("miner did not create snapshots before the forced-stop deadline")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)

    # Verify the recovered on-disk chain *before* mining another block:
    # a successful subsequent mine alone would not prove prior state replay.
    verified = run_node(binary, "testnet", "verify", "--data-dir", str(data_dir))
    recovered = re.search(r"height=(\d+) tip=([0-9a-f]{64})", verified)
    if recovered is None or int(recovered.group(1)) < saved_height:
        raise AssertionError(
            f"forced-stop recovery lost a visible completed snapshot "
            f"(observed height {saved_height}): {verified}"
        )
    recovered_height = int(recovered.group(1))
    # Compare against the identity captured *before* the forced stop. The
    # previous check rescanned files after the kill and could silently skip
    # the tip assertion if that manifest was no longer present.
    if recovered_height == saved_height and recovered.group(2) != saved_tip:
        raise AssertionError(
            f"recovery tip {recovered.group(2)} differs from pre-crash "
            f"snapshot tip {saved_tip} at height {saved_height}"
        )
    resumed = run_node(
        binary, "testnet", "mine", "--data-dir", str(data_dir),
        "--miner", miner, "--count", "1",
    )
    heights = [int(value) for value in re.findall(r"height=(\d+)\b", resumed)]
    if not heights or max(heights) <= recovered_height:
        raise AssertionError(
            f"node did not resume above the last committed snapshot (height {saved_height}):\n{resumed}"
        )
    return max(heights)


def offline_backup_validation_test(binary: Path, data_dir: Path) -> None:
    """Verify an offline copy; detect missing index pages and corrupted blocks."""
    # The miner has exited; never copy a live node's data files.
    source = run_node(binary, "testnet", "verify", "--data-dir", str(data_dir))
    source_state = re.search(r"height=(\d+) tip=([0-9a-f]{64})", source)
    if source_state is None or int(source_state.group(1)) < 2:
        raise AssertionError(f"source chain is not verifiable: {source}")

    def assert_not_equivalent(copy: Path, reason: str) -> None:
        try:
            incomplete = run_node(binary, "testnet", "verify", "--data-dir", str(copy))
        except RuntimeError:
            return  # A rejected copy cannot masquerade as the source.
        incomplete_state = re.search(r"height=(\d+) tip=([0-9a-f]{64})", incomplete)
        if incomplete_state is not None and incomplete_state.groups() == source_state.groups():
            raise AssertionError(
                f"{reason} backup was incorrectly accepted as the original chain"
            )

    backup = data_dir.with_name(f"{data_dir.name}-offline-copy")
    shutil.copytree(data_dir, backup)
    copied = run_node(binary, "testnet", "verify", "--data-dir", str(backup))
    copied_state = re.search(r"height=(\d+) tip=([0-9a-f]{64})", copied)
    if copied_state is None or copied_state.groups() != source_state.groups():
        raise AssertionError(
            f"offline copy differs from stopped source: {source} / {copied}"
        )

    # A v6 index manifest without its pages is an incomplete backup.
    pages = list((backup / "index-pages-v6").glob("*.aurpage"))
    if not pages:
        raise AssertionError("offline test snapshot unexpectedly has no v6 index pages")
    for page in pages:
        page.unlink()
    assert_not_equivalent(backup, "missing-index-pages")

    # A second, independent copy with a damaged immutable block must also
    # fail verification or fall back to a different, older valid state.
    damaged = data_dir.with_name(f"{data_dir.name}-corrupt-block-copy")
    shutil.copytree(data_dir, damaged)
    blocks = sorted((damaged / "blocks-v5").glob("*.aretblk"))
    if not blocks:
        raise AssertionError("offline test snapshot unexpectedly has no v5 blocks")
    first_block = blocks[0]
    raw = bytearray(first_block.read_bytes())
    if not raw:
        raise AssertionError("offline test block is unexpectedly empty")
    raw[-1] ^= 1
    first_block.write_bytes(raw)
    assert_not_equivalent(damaged, "corrupt-block")


def write_limit_recovery_test(binary: Path, data_dir: Path, miner: str) -> int | None:
    if os.name == "nt":
        return None
    import resource
    import signal

    run_node(
        binary, "testnet", "mine", "--data-dir", str(data_dir),
        "--miner", miner, "--count", "1",
    )

    def constrain_file_size() -> None:
        signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
        resource.setrlimit(resource.RLIMIT_FSIZE, (128, 128))

    process = subprocess.Popen(
        [str(binary), "testnet", "mine", "--data-dir", str(data_dir),
         "--miner", miner, "--count", "1"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        preexec_fn=constrain_file_size,
    )
    try:
        return_code = process.wait(timeout=30)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
        raise TimeoutError("size-limited snapshot write did not finish")
    if return_code == 0:
        raise AssertionError("size-limited snapshot unexpectedly succeeded")

    resumed = run_node(
        binary, "testnet", "mine", "--data-dir", str(data_dir),
        "--miner", miner, "--count", "1",
    )
    heights = [int(value) for value in re.findall(r"height=(\d+)\b", resumed)]
    if not heights or max(heights) != 2:
        raise AssertionError(
            f"node did not recover the last complete snapshot after a size-limit write failure:\n{resumed}"
        )
    return max(heights)


def main() -> int:
    subprocess.run([CARGO, "build", "--locked", "-p", "aretovus-node"], cwd=ROOT, check=True)
    subprocess.run([CARGO, "build", "--locked", "-p", "aretovus-wallet"], cwd=ROOT, check=True)
    suffix = ".exe" if os.name == "nt" else ""
    binary = ROOT / "target" / "debug" / f"aretovus-node{suffix}"
    wallet = ROOT / "target" / "debug" / f"aretovus-wallet{suffix}"
    if not binary.is_file() or not wallet.is_file():
        raise FileNotFoundError("built node or wallet executable is missing")

    with tempfile.TemporaryDirectory(prefix="aretovus-p2p-smoke-") as temporary:
        base = Path(temporary)
        node_a, node_b = base / "node-a", base / "node-b"
        keyfile = base / "miner.aretkey"
        key_output = run_wallet(wallet, "keygen", "--out", str(keyfile), "--plain-testnet")
        miner_match = re.search(r"address=([0-9a-f]{64})", key_output)
        if miner_match is None:
            raise AssertionError(f"wallet did not print its testnet address:\n{key_output}")
        miner = miner_match.group(1)
        addr_a, addr_b = free_address(), free_address()
        while addr_b[1] == addr_a[1]:
            addr_b = free_address()
        logs = [base / "a.log", base / "b.log"]
        servers: list[subprocess.Popen[str]] = []
        try:
            # Keep A offline while its miner writes; B serves a separate data directory.
            for data_dir, address, log in ((node_b, addr_b, logs[1]),):
                stream = log.open("w", encoding="utf-8")
                server = subprocess.Popen(
                    [str(binary), "testnet", "serve", "--data-dir", str(data_dir),
                     "--listen", f"{address[0]}:{address[1]}"],
                    cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
                )
                # Keep the parent stream alive until the server exits.
                server._aretovus_log_stream = stream  # type: ignore[attr-defined]
                servers.append(server)
                wait_listening(server, address, log)

            concurrent = subprocess.run(
                [str(binary), "testnet", "mine", "--data-dir", str(node_b),
                 "--miner", miner, "--count", "1"],
                cwd=ROOT, text=True, capture_output=True, timeout=10,
            )
            if concurrent.returncode == 0 or "already in use or cannot be locked" not in concurrent.stderr:
                raise AssertionError(f"second writer was not rejected: {concurrent.stdout}{concurrent.stderr}")

            mined = run_node(
                binary, "testnet", "mine", "--data-dir", str(node_a),
                "--miner", miner, "--count", "3", "--peer", f"{addr_b[0]}:{addr_b[1]}",
            )
            mined_tip = re.search(r"height=3\b.*\btip=([0-9a-f]{64})", mined)
            if mined_tip is None:
                raise AssertionError(f"miner did not report the expected three-block tip:\n{mined}")
            cap_units = 500_000_000 * 100_000_000
            expected_issued = 0
            expected_miner = 0
            expected_rows = []
            for height in range(1, 4):
                remaining = cap_units - expected_issued
                issuance = min(remaining, max(1, remaining // 1_000_000))
                expected_issued += issuance
                expected_miner += issuance - issuance // 20
                expected_rows.append((height, expected_issued, expected_miner))
            mined_rows = [
                (int(height), int(issued), int(balance))
                for height, issued, balance in re.findall(
                    r"height=(\d+) tip=[0-9a-f]{64} .*issued_units=(\d+) miner_balance_units=(\d+)",
                    mined,
                )
            ]
            if mined_rows != expected_rows:
                raise AssertionError(
                    f"mined blocks do not follow smooth emission and 95/5 split:\n{mined}"
                )

            # Recover an already funded account after deleting its original key file.
            recovery_file = base / "offline-recovery.txt"
            balance_before_loss = run_wallet(
                wallet, "balance", "--key-file", str(keyfile),
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            refused_export = subprocess.run(
                [str(wallet), "recovery-export", "--key-file", str(keyfile),
                 "--out", str(recovery_file)],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if refused_export.returncode == 0 or recovery_file.exists():
                raise AssertionError("recovery export did not require secret acknowledgement")
            exported = run_wallet(
                wallet, "recovery-export", "--key-file", str(keyfile),
                "--out", str(recovery_file), "--acknowledge-secret",
            )
            if recovery_file.read_text().strip() in exported:
                raise AssertionError("recovery secret leaked to command output")
            run_wallet(wallet, "recovery-verify", "--from", str(recovery_file),
                       "--expected-account", miner)
            bad_restore = base / "wrong-account.aretkey"
            refused_restore = subprocess.run(
                [str(wallet), "recovery-restore", "--from", str(recovery_file),
                 "--out", str(bad_restore), "--expected-account", RECIPIENT,
                 "--plain-testnet"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if refused_restore.returncode == 0 or bad_restore.exists():
                raise AssertionError("recovery wrote a wallet for the wrong expected account")
            keyfile.unlink()
            restored_key = run_wallet(
                wallet, "recovery-restore", "--from", str(recovery_file),
                "--out", str(keyfile), "--expected-account", miner, "--plain-testnet",
            )
            if f"account={miner}" not in restored_key:
                raise AssertionError("recovery changed the funded account identity")
            balance_after_loss = run_wallet(
                wallet, "balance", "--key-file", str(keyfile),
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            if balance_before_loss != balance_after_loss:
                raise AssertionError("recovery changed the funded balance or nonce")

            # Preview, voluntary limits and cancellation must leave pending state untouched.
            payment_args = [
                "--key-file", str(keyfile), "--to", RECIPIENT,
                "--amount", "1.00000000", "--fee", "0.00000003",
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            ]
            before_review = run_wallet(
                wallet, "balance", "--key-file", str(keyfile),
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            pending_nonce_match = re.search(r"next_nonce=(\d+)", before_review)
            if pending_nonce_match is None:
                raise AssertionError("wallet balance does not include next nonce")
            review_nonce = pending_nonce_match.group(1)
            run_wallet(
                wallet, "limits-create", "--key-file", str(keyfile),
                "--max-amount", "1.00000000", "--max-fee", "0.00000003",
            )
            saved_limits = run_wallet(wallet, "limits-show", "--key-file", str(keyfile))
            if "limits=active" not in saved_limits or "max_amount=1.00000000" not in saved_limits:
                raise AssertionError(f"wallet did not reload its limits: {saved_limits}")
            policy_file = Path(str(keyfile) + ".limits")
            original_policy = policy_file.read_text()
            for name, value, expected_error in [
                ("--amount", "1.00000001", "voluntary amount limit"),
                ("--fee", "0.00000004", "voluntary fee limit"),
            ]:
                excessive = list(payment_args)
                excessive[excessive.index(name) + 1] = value
                blocked = subprocess.run(
                    [str(wallet), "send", *excessive, "--yes",
                     "--max-amount", "100", "--max-fee", "1"],
                    cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
                )
                if blocked.returncode == 0 or expected_error not in blocked.stderr:
                    raise AssertionError("one-shot flags bypassed persistent wallet limits")
            policy_file.write_text("invalid-policy\n")
            malformed = subprocess.run(
                [str(wallet), "send", *payment_args, "--yes"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if malformed.returncode == 0 or "invalid wallet limits" not in malformed.stderr:
                raise AssertionError("malformed wallet policy did not block sending")
            policy_file.write_text(original_policy)

            preview = run_wallet(wallet, "send", *payment_args, "--dry-run")
            if ("preview_only:" not in preview or "transaction_id=" in preview
                    or "effective_max_amount=1.00000000" not in preview):
                raise AssertionError(f"preview unexpectedly submitted a payment: {preview}")
            for extra, reply, expected_error in [
                ([], "", "payment cancelled before signing"),
                ([], "SEND " + "44" * 32 + "\n", "payment cancelled before signing"),
                (["--yes", "--max-amount", "0.99999999"], "", "voluntary amount limit"),
                (["--yes", "--max-fee", "0.00000002"], "", "voluntary fee limit"),
                (["--yes", "--dry-run"], "", "cannot be combined"),
            ]:
                rejected = subprocess.run(
                    [str(wallet), "send", *payment_args, *extra],
                    cwd=ROOT, text=True, input=reply, capture_output=True, timeout=20,
                )
                if rejected.returncode == 0 or expected_error not in rejected.stderr:
                    raise AssertionError(f"unsafe payment was not rejected: {rejected.stdout}{rejected.stderr}")
            # A policy changed while the user reviews must cancel before signing.
            reviewing = subprocess.Popen(
                [str(wallet), "send", *payment_args],
                cwd=ROOT, text=True, stdin=subprocess.PIPE,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            try:
                while True:
                    line = reviewing.stdout.readline()
                    if not line:
                        raise AssertionError("payment exited before showing confirmation")
                    if "A confirmed payment is irreversible." in line:
                        break
                policy_file.write_text(original_policy.replace(
                    "max_amount=1.00000000", "max_amount=0.50000000"
                ))
                output, errors = reviewing.communicate(
                    input=f"SEND ARETOVUS-TESTNET {RECIPIENT} 1.00000000 0.00000003 {review_nonce}\n",
                    timeout=20,
                )
                if reviewing.returncode == 0 or "wallet limits changed during review" not in errors:
                    raise AssertionError(f"changed policy was not detected: {output}{errors}")
            finally:
                if reviewing.poll() is None:
                    reviewing.kill()
                    reviewing.wait(timeout=5)
                policy_file.write_text(original_policy)

            after_review = run_wallet(
                wallet, "balance", "--key-file", str(keyfile),
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            if before_review != after_review:
                raise AssertionError("preview/rejected payments changed balance or nonce")

            # Sign without a node, then broadcast while the key file is unavailable.
            public_account = run_wallet(
                wallet, "account", "--address", miner,
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            nonce_match = re.search(r"next_nonce=(\d+)", public_account)
            if nonce_match is None:
                raise AssertionError("wallet did not report a nonce for offline preparation")
            signed_file = base / "signed-payment.aurtx"
            signing_args = [
                "--key-file", str(keyfile), "--to", RECIPIENT,
                "--amount", "1.00000000", "--fee", "0.00000003",
                "--nonce", nonce_match.group(1), "--out", str(signed_file),
            ]
            for extra, expected_error in [
                ([], "offline payment cancelled before signing"),
                (["--yes", "--node", "127.0.0.1:1"], "unsupported option"),
                (["--yes", "--nonce", "99"], "duplicate command option"),
            ]:
                refused = subprocess.run(
                    [str(wallet), "sign-offline", *signing_args, *extra],
                    cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
                )
                if refused.returncode == 0 or expected_error not in refused.stderr or signed_file.exists():
                    raise AssertionError(f"unsafe offline signing was allowed: {refused.stderr}")
            excessive = list(signing_args)
            excessive[excessive.index("--amount") + 1] = "1.00000001"
            refused = subprocess.run(
                [str(wallet), "sign-offline", *excessive, "--yes", "--max-amount", "100"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if refused.returncode == 0 or "voluntary amount limit" not in refused.stderr or signed_file.exists():
                raise AssertionError("offline signing bypassed persistent limits")
            preview = run_wallet(wallet, "sign-offline", *signing_args, "--dry-run")
            if "preview_only:" not in preview or signed_file.exists():
                raise AssertionError("offline preview wrote a signed file")
            authorized = subprocess.run(
                [str(wallet), "sign-offline", *signing_args],
                cwd=ROOT, text=True,
                input=f"SEND ARETOVUS-TESTNET {RECIPIENT} 1.00000000 0.00000003 {nonce_match.group(1)}\n",
                capture_output=True, timeout=20,
            )
            prepared = authorized.stdout + authorized.stderr
            if authorized.returncode or not signed_file.is_file():
                raise AssertionError(f"offline signing failed: {prepared}")
            prepared_match = re.search(r"prepared_txid=([0-9a-f]{64})", prepared)
            if prepared_match is None or "network_requests=none" not in prepared:
                raise AssertionError("offline signer did not identify the prepared transaction")
            inspected = run_wallet(
                wallet, "inspect-signed", "--from", str(signed_file),
            )
            for expected in [
                "verified_signed_transaction=true",
                "network=ARETOVUS-TESTNET version=3",
                f"txid={prepared_match.group(1)}",
                f"recipient={RECIPIENT}",
                "amount=1.00000000 fee=0.00000003",
                f"nonce={nonce_match.group(1)}",
                "read_only=true private_key_loaded=false network_requests=none",
            ]:
                if expected not in inspected:
                    raise AssertionError(f"offline inspection omitted {expected}: {inspected}")
            # Verify the exact externally supplied intent without loading any key or node.
            expected_intent_args = [
                "--from", str(signed_file),
                "--sender", miner,
                "--to", RECIPIENT,
                "--amount", "1.00000000",
                "--fee", "0.00000003",
                "--nonce", nonce_match.group(1),
            ]
            approved_intent = run_wallet(wallet, "verify-signed-intent", *expected_intent_args)
            if (f"verified_signed_intent=true txid={prepared_match.group(1)}" not in approved_intent
                    or "private_key_loaded=false network_requests=none" not in approved_intent):
                raise AssertionError(f"verifier did not bind the exact expected intent: {approved_intent}")
            for option_name, changed_value in [
                ("--sender", "33" * 32),
                ("--to", "44" * 32),
                ("--amount", "1.00000001"),
                ("--fee", "0.00000004"),
                ("--nonce", str(int(nonce_match.group(1)) + 1)),
            ]:
                substituted = list(expected_intent_args)
                substituted[substituted.index(option_name) + 1] = changed_value
                refused = subprocess.run(
                    [str(wallet), "verify-signed-intent", *substituted],
                    cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
                )
                if (refused.returncode == 0 or "verified_signed_intent=true" in refused.stdout
                        or "reviewed payment intent" not in refused.stderr):
                    raise AssertionError(f"verifier allowed changed {option_name}: {refused.stdout} {refused.stderr}")
            for forbidden in ["--yes", "--node", "127.0.0.1:1"]:
                refused = subprocess.run(
                    [str(wallet), "verify-signed-intent", *expected_intent_args, forbidden],
                    cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
                )
                if refused.returncode == 0 or "verified_signed_intent=true" in refused.stdout:
                    raise AssertionError("signed-intent verifier accepted an unsupported option")

            invalid_inspect = subprocess.run(
                [str(wallet), "inspect-signed", "--from", str(signed_file),
                 "--node", "127.0.0.1:1"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if invalid_inspect.returncode == 0 or "unsupported option" not in invalid_inspect.stderr:
                raise AssertionError("signed-file inspector accepted a network argument")

            balance_after_signing = run_wallet(
                wallet, "balance", "--key-file", str(keyfile),
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
            if balance_after_signing != before_review:
                raise AssertionError("offline preparation changed balance or nonce")

            damaged = base / "damaged-payment.aurtx"
            damaged_bytes = bytearray(signed_file.read_bytes())
            damaged_bytes[-1] ^= 1
            damaged.write_bytes(damaged_bytes)
            rejected_inspect = subprocess.run(
                [str(wallet), "inspect-signed", "--from", str(damaged)],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if rejected_inspect.returncode == 0 or "verified_signed_transaction=true" in rejected_inspect.stdout:
                raise AssertionError("inspector accepted a corrupted signature")
            invalid_intent_args = list(expected_intent_args)
            invalid_intent_args[invalid_intent_args.index("--from") + 1] = str(damaged)
            invalid_intent = subprocess.run(
                [str(wallet), "verify-signed-intent", *invalid_intent_args],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if invalid_intent.returncode == 0 or "verified_signed_intent=true" in invalid_intent.stdout:
                raise AssertionError("intent verifier accepted a damaged signed file")
            rejected_broadcast = subprocess.run(
                [str(wallet), "broadcast", "--from", str(damaged),
                 "--node", f"{addr_b[0]}:{addr_b[1]}", "--yes"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if rejected_broadcast.returncode == 0:
                raise AssertionError("broadcast accepted a corrupted signed payment")
            override = subprocess.run(
                [str(wallet), "broadcast", "--from", str(signed_file),
                 "--node", f"{addr_b[0]}:{addr_b[1]}", "--to", RECIPIENT, "--yes"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if override.returncode == 0 or "unsupported option" not in override.stderr:
                raise AssertionError("broadcast silently accepted signed-field overrides")
            # Adversarial file substitution: a second *validly signed* payment
            # must not replace the transaction reviewed on the offline machine.
            swapped_file = base / "swapped-payment.aurtx"
            swapped_args = list(signing_args)
            swapped_args[swapped_args.index("--to") + 1] = "44" * 32
            swapped_args[swapped_args.index("--out") + 1] = str(swapped_file)
            run_wallet(wallet, "sign-offline", *swapped_args, "--yes")
            swapped_details = run_wallet(wallet, "inspect-signed", "--from", str(swapped_file))
            if (f"recipient={'44' * 32}" not in swapped_details
                    or f"txid={prepared_match.group(1)}" in swapped_details):
                raise AssertionError("adversarial signed payment was not independently different")
            # Replace the *same reviewed file path* with a different valid
            # authorization, then restore the original bytes for the good path.
            reviewed_bytes = signed_file.read_bytes()
            signed_file.write_bytes(swapped_file.read_bytes())
            try:
                rejected_swap = subprocess.run(
                    [str(wallet), "broadcast", "--from", str(signed_file),
                     "--node", "127.0.0.1:1",
                     "--expected-txid", prepared_match.group(1), "--yes"],
                    cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
                )
            finally:
                signed_file.write_bytes(reviewed_bytes)
            if (rejected_swap.returncode == 0
                    or "signed payment differs from externally reviewed txid" not in rejected_swap.stderr
                    or "transaction_id=" in rejected_swap.stdout):
                raise AssertionError(
                    f"file substitution was not rejected before RPC: {rejected_swap.stdout} {rejected_swap.stderr}"
                )
            invalid_pin = subprocess.run(
                [str(wallet), "broadcast", "--from", str(signed_file),
                 "--node", "127.0.0.1:1",
                 "--expected-txid", "not-a-txid", "--yes"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if (invalid_pin.returncode == 0
                    or "transaction_id=" in invalid_pin.stdout):
                raise AssertionError("noncanonical pinned txid was accepted")

            cancelled = subprocess.run(
                [str(wallet), "broadcast", "--from", str(signed_file),
                 "--node", f"{addr_b[0]}:{addr_b[1]}"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if cancelled.returncode == 0 or "broadcast cancelled" not in cancelled.stderr:
                raise AssertionError("broadcast did not require explicit authorization")

            unavailable_key = base / "offline-key-unavailable.aretkey"
            keyfile.rename(unavailable_key)
            try:
                offline_review = run_wallet(
                    wallet, "inspect-signed", "--from", str(signed_file),
                )
                if offline_review != inspected:
                    raise AssertionError("inspection output depended on the availability of the private key")
                offline_intent = run_wallet(wallet, "verify-signed-intent", *expected_intent_args)
                if offline_intent != approved_intent:
                    raise AssertionError("signed-intent verification depended on the private key file")
                public_without_key = run_wallet(
                    wallet, "account", "--address", miner,
                    "--node", f"{addr_b[0]}:{addr_b[1]}",
                )
                if public_without_key != public_account:
                    raise AssertionError("keyless public account query changed before broadcast")
                broadcast = subprocess.run(
                    [str(wallet), "broadcast", "--from", str(signed_file),
                     "--node", f"{addr_b[0]}:{addr_b[1]}",
                     "--expected-txid", prepared_match.group(1)],
                    cwd=ROOT, text=True,
                    input=f"SEND ARETOVUS-TESTNET {RECIPIENT} 1.00000000 0.00000003 {nonce_match.group(1)}\n",
                    capture_output=True, timeout=20,
                )
                sent = broadcast.stdout + broadcast.stderr
                if broadcast.returncode:
                    raise AssertionError(f"broadcast without a key failed: {sent}")
            finally:
                unavailable_key.rename(keyfile)
            if (f"transaction_id={prepared_match.group(1)}" not in sent
                    or "external_txid_pin_verified=true" not in sent):
                raise AssertionError("broadcast did not preserve the externally reviewed transaction")
            duplicate = subprocess.run(
                [str(wallet), "broadcast", "--from", str(signed_file),
                 "--node", f"{addr_b[0]}:{addr_b[1]}", "--yes"],
                cwd=ROOT, text=True, input="", capture_output=True, timeout=20,
            )
            if duplicate.returncode == 0 or "signed nonce differs" not in duplicate.stderr:
                raise AssertionError("broadcast did not detect a consumed nonce")
            tx_match = re.search(r"transaction_id=([0-9a-f]{64})", sent)
            if tx_match is None or "mempool=pending" not in sent:
                raise AssertionError(f"wallet did not submit transaction to mempool:\n{sent}")
            pending = run_wallet(
                wallet, "status", "--node", f"{addr_b[0]}:{addr_b[1]}",
                "--txid", tx_match.group(1),
            )
            if "Pending" not in pending:
                raise AssertionError(f"submitted transaction was not pending:\n{pending}")

            mined_payment = run_node(
                binary, "testnet", "mine", "--data-dir", str(node_a),
                "--miner", miner, "--count", "1", "--peer", f"{addr_b[0]}:{addr_b[1]}",
            )
            if not re.search(r"height=4\b.*transactions=1", mined_payment):
                raise AssertionError(f"miner did not include the pending wallet transaction:\n{mined_payment}")
            remaining = cap_units - expected_issued
            issuance = min(remaining, max(1, remaining // 1_000_000))
            expected_issued += issuance
            expected_miner += issuance - issuance // 20
            expected_miner -= 100_000_000 + 3
            expected_miner += 1
            payment_row = re.search(
                r"height=4 tip=[0-9a-f]{64} .*issued_units=(\d+) miner_balance_units=(\d+)",
                mined_payment,
            )
            if payment_row is None or (int(payment_row.group(1)), int(payment_row.group(2))) != (
                expected_issued, expected_miner
            ):
                raise AssertionError(f"mined wallet payment reward accounting was incorrect:\n{mined_payment}")
            confirmed = run_wallet(
                wallet, "status", "--node", f"{addr_b[0]}:{addr_b[1]}",
                "--txid", tx_match.group(1),
            )
            if "Confirmed" not in confirmed:
                raise AssertionError(f"transaction was not confirmed after mining:\n{confirmed}")
            verified = run_wallet(
                wallet, "verify", "--node", f"{addr_b[0]}:{addr_b[1]}",
                "--txid", tx_match.group(1), "--confirmations", "1",
            )
            if "SPV verified:" not in verified or "confirmations=1" not in verified:
                raise AssertionError(f"SPV transaction verification failed:\n{verified}")

            # Stop B before a separate process reads and extends its persisted chain.
            for server in servers:
                server.terminate()
                server.wait(timeout=5)
            stream = logs[0].open("a", encoding="utf-8")
            server_a = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(node_a),
                 "--listen", f"{addr_a[0]}:{addr_a[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_a._aretovus_log_stream = stream  # type: ignore[attr-defined]
            servers.append(server_a)
            wait_listening(server_a, addr_a, logs[0])

            fresh_node = base / "node-fresh"
            synced = run_node(
                binary, "testnet", "sync", "--data-dir", str(fresh_node),
                "--peer", f"{addr_a[0]}:{addr_a[1]}",
            )
            tips = re.findall(r"height=(\d+) tip=([0-9a-f]{64})", synced)
            accepted = re.search(r"accepted=(\d+)", synced)
            if not tips or tips[-1][0] != "4" or accepted is None or accepted.group(1) != "4":
                raise AssertionError(f"fresh node did not pull all four blocks:\n{synced}")
            payment_tip = re.search(r"height=4\b.*\btip=([0-9a-f]{64})", mined_payment)
            if payment_tip is None or tips[-1][1] != payment_tip.group(1):
                raise AssertionError(f"fresh node did not converge to the latest mined tip:\n{mined_payment}\n{synced}")

            # A separate process reopens B's persisted snapshot and checks the same peer tip.
            restored = run_node(
                binary, "testnet", "sync", "--data-dir", str(node_b),
                "--peer", f"{addr_a[0]}:{addr_a[1]}",
            )
            restored_tips = re.findall(r"height=(\d+) tip=([0-9a-f]{64})", restored)
            if not restored_tips or restored_tips[-1] != tips[-1]:
                raise AssertionError(f"pushed node did not restore the same tip:\n{restored}")
            # Extend past two retarget boundaries, push to A, restart A, and pull the longer chain.
            extended = run_node(
                binary, "testnet", "mine", "--data-dir", str(node_b),
                "--miner", miner, "--count", "16", "--peer", f"{addr_a[0]}:{addr_a[1]}",
            )
            extended_tip_match = re.search(r"height=20\b.*\btip=([0-9a-f]{64})", extended)
            if extended_tip_match is None:
                raise AssertionError(f"extended testnet did not pass retarget windows:\n{extended}")

            server_a.terminate()
            server_a.wait(timeout=5)
            stream = logs[0].open("a", encoding="utf-8")
            server_a = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(node_a),
                 "--listen", f"{addr_a[0]}:{addr_a[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_a._aretovus_log_stream = stream  # type: ignore[attr-defined]
            servers.append(server_a)
            wait_listening(server_a, addr_a, logs[0])

            long_sync = run_node(
                binary, "testnet", "sync", "--data-dir", str(fresh_node),
                "--peer", f"{addr_a[0]}:{addr_a[1]}",
            )
            long_tips = re.findall(r"height=(\d+) tip=([0-9a-f]{64})", long_sync)
            long_accepted = re.search(r"accepted=(\d+)", long_sync)
            if (
                not long_tips
                or long_tips[-1][0] != "20"
                or long_tips[-1][1] != extended_tip_match.group(1)
                or long_accepted is None
                or long_accepted.group(1) != "16"
            ):
                raise AssertionError(f"fresh node did not recover and sync the 20-block chain:\n{long_sync}")

            fork_node = base / "node-fork"
            competing = run_node(
                binary, "testnet", "mine", "--data-dir", str(fork_node),
                "--miner", miner, "--count", "21",
            )
            competing_tip = re.search(
                r"height=21\b.*\btip=([0-9a-f]{64})", competing
            )
            if competing_tip is None:
                raise AssertionError(f"competing branch did not reach height 21:\n{competing}")

            addr_c = free_address()
            while addr_c[1] in (addr_a[1], addr_b[1]):
                addr_c = free_address()
            fork_log = base / "fork.log"
            stream = fork_log.open("w", encoding="utf-8")
            server_c = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(fork_node),
                 "--listen", f"{addr_c[0]}:{addr_c[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_c._aretovus_log_stream = stream  # type: ignore[attr-defined]
            servers.append(server_c)
            wait_listening(server_c, addr_c, fork_log)

            server_a.terminate()
            server_a.wait(timeout=5)
            reorg = run_node(
                binary, "testnet", "sync", "--data-dir", str(node_a),
                "--peer", f"{addr_c[0]}:{addr_c[1]}",
            )
            reorg_tips = re.findall(r"height=(\d+) tip=([0-9a-f]{64})", reorg)
            if not reorg_tips or reorg_tips[-1] != ("21", competing_tip.group(1)):
                raise AssertionError(f"node did not reorganize to the heavier peer branch:\n{reorg}")

            stream = logs[0].open("a", encoding="utf-8")
            server_a = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(node_a),
                 "--listen", f"{addr_a[0]}:{addr_a[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_a._aretovus_log_stream = stream  # type: ignore[attr-defined]
            servers.append(server_a)
            wait_listening(server_a, addr_a, logs[0])

            converged = run_node(
                binary, "testnet", "sync", "--data-dir", str(fresh_node),
                "--peer", f"{addr_a[0]}:{addr_a[1]}",
            )
            converged_tips = re.findall(r"height=(\d+) tip=([0-9a-f]{64})", converged)
            if not converged_tips or converged_tips[-1] != ("21", competing_tip.group(1)):
                raise AssertionError(f"fresh peer did not converge after the reorganization:\n{converged}")
            post_reorg_status = run_wallet(
                wallet, "status", "--node", f"{addr_a[0]}:{addr_a[1]}",
                "--txid", tx_match.group(1),
            )
            if "Confirmed" in post_reorg_status:
                raise AssertionError("wallet still reports the orphaned transaction as confirmed")

            # A running peer must catch up without a manual sync command.
            addr_d = free_address()
            while addr_d[1] in (addr_a[1], addr_b[1], addr_c[1]):
                addr_d = free_address()
            live_node = base / "node-live"
            live_log = base / "live.log"
            stream = live_log.open("w", encoding="utf-8")
            server_d = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(live_node),
                 "--listen", f"{addr_d[0]}:{addr_d[1]}",
                 "--peer", f"{addr_c[0]}:{addr_c[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_d._aretovus_log_stream = stream  # type: ignore[attr-defined]
            servers.append(server_d)
            wait_listening(server_d, addr_d, live_log)

            def wait_live_tip(height: int, tip: str) -> None:
                deadline = time.monotonic() + 30
                last = ""
                while time.monotonic() < deadline:
                    if server_d.poll() is not None:
                        raise RuntimeError(f"background-sync peer exited: {live_log.read_text(errors='replace')}")
                    last = run_node(binary, "testnet", "verify", "--data-dir", str(live_node))
                    if f"height={height} tip={tip}" in last:
                        return
                    time.sleep(0.2)
                raise TimeoutError(
                    f"background-sync peer did not reach {height}/{tip}: {last}\n"
                    f"log: {live_log.read_text(errors='replace')}"
                )

            wait_live_tip(21, competing_tip.group(1))
            active_miner = base / "node-active-miner"
            run_node(
                binary, "testnet", "sync", "--data-dir", str(active_miner),
                "--peer", f"{addr_c[0]}:{addr_c[1]}",
            )
            next_block = run_node(
                binary, "testnet", "mine", "--data-dir", str(active_miner),
                "--miner", miner, "--count", "1",
                "--peer", f"{addr_c[0]}:{addr_c[1]}",
            )
            next_tip = re.search(r"height=22\b.*\btip=([0-9a-f]{64})", next_block)
            if next_tip is None:
                raise AssertionError(f"active miner did not extend to height 22:\n{next_block}")
            wait_live_tip(22, next_tip.group(1))

            # A wallet submits to C; D must learn the pending transaction via
            # its configured peer and serve it to a miner without contacting C.
            relayed = run_wallet(
                wallet, "send", "--yes", "--key-file", str(keyfile), "--to", RECIPIENT,
                "--amount", "1.00000000", "--fee", "0.00000003",
                "--node", f"{addr_c[0]}:{addr_c[1]}",
            )
            relay_match = re.search(r"transaction_id=([0-9a-f]{64})", relayed)
            if relay_match is None:
                raise AssertionError(f"source peer rejected relayed wallet transaction:\n{relayed}")
            relay_id = relay_match.group(1)
            deadline = time.monotonic() + 35
            while time.monotonic() < deadline:
                relay_status = run_wallet(
                    wallet, "status", "--node", f"{addr_d[0]}:{addr_d[1]}",
                    "--txid", relay_id,
                )
                if "Pending" in relay_status:
                    break
                time.sleep(0.5)
            else:
                raise TimeoutError(f"live peer did not relay pending transaction: {relay_status}")
            relayed_block = run_node(
                binary, "testnet", "mine", "--data-dir", str(active_miner),
                "--miner", miner, "--count", "1",
                "--peer", f"{addr_d[0]}:{addr_d[1]}",
            )
            if not re.search(r"height=23\b.*transactions=1", relayed_block):
                raise AssertionError(f"miner did not include relayed transaction:\n{relayed_block}")
            relayed_confirmed = run_wallet(
                wallet, "status", "--node", f"{addr_d[0]}:{addr_d[1]}",
                "--txid", relay_id,
            )
            if "Confirmed" not in relayed_confirmed:
                raise AssertionError("gossip receiver did not confirm relayed transaction")

            crash_height = crash_recovery_test(binary, base / "crash-node", miner)
            offline_backup_validation_test(binary, base / "crash-node")
            size_limit_height = write_limit_recovery_test(
                binary, base / "write-limit-node", miner
            )
            print(
                "offline signing and keyless broadcast, persistent wallet limits, lost-key recovery, wallet, SPV, 20-block soak, heavier-branch reorganization, live peer catch-up, mempool relay, forced-stop "
                "offline backup verification and write-limit recovery passed: "
                f"reorg_height=21 tip={competing_tip.group(1)} "
                f"crash_recovery_height={crash_height} write_limit_height={size_limit_height}"
            )
        finally:
            for server in servers:
                if server.poll() is None:
                    server.terminate()
            for server in servers:
                try:
                    server.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)
                stream = getattr(server, "_aretovus_log_stream", None)
                if stream is not None:
                    stream.close()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"testnet smoke failed: {error}", file=sys.stderr)
        raise
