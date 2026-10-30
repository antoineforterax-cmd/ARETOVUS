#!/usr/bin/env python3
"""Verify the *packaged* AURORA Windows node across two loopback processes.

This is not a two-computer test: both nodes share one Windows CI runner.
No wallet secrets, real funds, network configuration, or external hosts.
"""
from __future__ import annotations

import argparse
import re
import socket
import subprocess
import tempfile
import time
from pathlib import Path

TIP = re.compile(
    r"verified testnet data: height=(\d+) tip=([0-9a-fA-F]{64}) "
    r"known_blocks=(\d+) work="
)


def run(*args: object, timeout: float = 90) -> str:
    completed = subprocess.run(
        [str(arg) for arg in args],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout,
        check=False,
    )
    if completed.returncode:
        raise RuntimeError(
            f"Command failed ({completed.returncode}): {args[0:3]}\n"
            + completed.stdout[-3000:]
        )
    return completed.stdout


def wallet_account(wallet: Path, keyfile: Path) -> str:
    # The wallet prints a warning on stderr for ephemeral plain-testnet
    # keys; parse only the 64-hex account line from the combined output.
    output = run(wallet, "address", "--key-file", keyfile)
    matches = re.findall(r"(?m)^([0-9a-fA-F]{64})$", output)
    if len(matches) != 1:
        raise AssertionError(f"Unexpected testnet wallet address output: {output[-300:]}")
    return matches[0].lower()


def await_pending(wallet: Path, address: str, txid: str, timeout: float = 55) -> None:
    """Require a peer to see a pending transaction before any mining."""
    started = time.monotonic()
    observed = "not yet available"
    while time.monotonic() - started < timeout:
        observed = run(
            wallet, "status", "--node", address, "--txid", txid
        ).strip()
        if observed == "Pending":
            return
        if observed != "Unknown":
            raise AssertionError(
                f"Unexpected transaction status before mining: {observed[-300:]}"
            )
        time.sleep(0.5)
    raise TimeoutError(
        f"Mempool relay to {address} failed for {txid}: {observed}"
    )



def expect_rejected_payment(
    wallet: Path, keyfile: Path, recipient: str, amount: str,
    node_address: str, expected_error: str,
) -> None:
    """A malformed or unaffordable payment must fail before signing/broadcast."""
    attempt = subprocess.run(
        [
            str(wallet), "send", "--key-file", str(keyfile),
            "--to", recipient, "--amount", amount, "--fee", "0.01",
            "--node", node_address, "--dry-run",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=30,
        check=False,
    )
    if attempt.returncode == 0 or expected_error not in attempt.stdout:
        raise AssertionError(
            "Wallet rejection missing or incorrect: "
            f"expected={expected_error!r} exit={attempt.returncode} "
            f"output={attempt.stdout[-800:]}"
        )
    if re.search(r"(?m)^transaction_id=", attempt.stdout):
        raise AssertionError("A rejected wallet payment returned a transaction ID")


def account_state(wallet: Path, keyfile: Path, node_address: str) -> str:
    output = run(wallet, "balance", "--key-file", keyfile, "--node", node_address)
    match = re.search(
        r"(?m)^address=[0-9a-fA-F]{64} pending_adjusted_balance=[0-9.]+ next_nonce=\\d+$",
        output,
    )
    if not match:
        raise AssertionError(f"Unexpected wallet state: {output[-800:]}")
    return match.group(0)


def choose_port() -> int:
    # Reserve only briefly. All sockets remain bound to loopback.
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def await_listener(proc: subprocess.Popen[bytes], port: int, log: Path) -> None:
    end = time.monotonic() + 20
    while time.monotonic() < end:
        if proc.poll() is not None:
            raise RuntimeError(
                f"Node exited before listening on {port}:\n"
                + log.read_text(errors="replace")[-3000:]
            )
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                return
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(f"Node did not listen on {port}: {log.read_text(errors='replace')[-3000:]}")


def start_server(
    node: Path, path: Path, port: int, log: Path, peer: int | None = None
) -> subprocess.Popen[bytes]:
    command = [
        str(node), "testnet", "serve", "--data-dir", str(path),
        "--listen", f"127.0.0.1:{port}",
    ]
    if peer is not None:
        command += ["--peer", f"127.0.0.1:{peer}"]
    with log.open("wb") as stream:
        process = subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT)
    await_listener(process, port, log)
    return process


def stop(process: subprocess.Popen[bytes] | None) -> None:
    if process is not None and process.poll() is None:
        # Deliberately abrupt: verify snapshot recovery after an unclean exit.
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=10)


def state(node: Path, directory: Path) -> tuple[int, str, int, str]:
    verified = run(node, "testnet", "verify", "--data-dir", directory)
    match = TIP.search(verified)
    if not match:
        raise AssertionError(f"Unexpected verify output: {verified[-1000:]}")
    return int(match[1]), match[2].lower(), int(match[3]), verified.strip()


def await_tip(
    node: Path,
    directory: Path,
    expected: tuple[int, str, int, str],
    timeout: float = 55,
) -> float:
    begin = time.monotonic()
    observed = "not yet available"
    while time.monotonic() - begin < timeout:
        try:
            now = state(node, directory)
            observed = now[3]
            if now[0] == expected[0] and now[1] == expected[1]:
                if now[2] != expected[2]:
                    raise AssertionError(
                        f"Same tip but inconsistent known_blocks: {now[2]} != {expected[2]}"
                    )
                return time.monotonic() - begin
        except (OSError, RuntimeError):
            pass
        time.sleep(0.3)
    raise TimeoutError(
        f"Two-node convergence failed: expected {expected[3]}, observed {observed}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package-root", type=Path, required=True)
    args = parser.parse_args()
    package = args.package_root.resolve()
    node = package / "bin" / "aurora-node.exe"
    if not node.is_file():
        raise FileNotFoundError(node)
    wallet = package / "bin" / "aurora-wallet.exe"
    if not wallet.is_file():
        raise FileNotFoundError(wallet)
    with tempfile.TemporaryDirectory(prefix="aurora-packaged-two-nodes-") as temp:
        root = Path(temp)
        a_data, b_data, miner_data = (
            root / "server-a", root / "server-b", root / "miner"
        )
        a_port, b_port = choose_port(), choose_port()
        while b_port == a_port:
            b_port = choose_port()
        a = b = None
        try:
            # These disposable, unencrypted keys exist only within the
            # temporary CI directory. Never use this option with real funds.
            alice_key = root / "alice.aurkey"
            bob_key = root / "bob.aurkey"
            run(wallet, "keygen", "--out", alice_key, "--plain-testnet")
            run(wallet, "keygen", "--out", bob_key, "--plain-testnet")
            miner = wallet_account(wallet, alice_key)
            bob = wallet_account(wallet, bob_key)

            # Both servers must pull their peer mempool, in both directions.
            a = start_server(node, a_data, a_port, root / "server-a.log", b_port)
            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1", "--peer", f"127.0.0.1:{a_port}"
            )
            source1 = state(node, a_data)
            if source1[0] != 1:
                raise AssertionError(f"Mine/propagation failed: {source1[3]}")
            b = start_server(node, b_data, b_port, root / "server-b.log", a_port)
            initial = await_tip(node, b_data, source1)
            stop(b)
            b = None

            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1", "--peer", f"127.0.0.1:{a_port}"
            )
            source2 = state(node, a_data)
            if source2[0] != 2 or source2[1] == source1[1]:
                raise AssertionError(f"Second block did not propagate: {source2[3]}")
            b = start_server(node, b_data, b_port, root / "server-b-restart.log", a_port)
            resumed = await_tip(node, b_data, source2)
            # Exercise the *packaged* ML-DSA wallet, not just node block
            # propagation: sign locally, submit via A, mine and verify SPV
            # inclusion independently through A and B.
            a_address = f"127.0.0.1:{a_port}"
            b_address = f"127.0.0.1:{b_port}"
            sent = run(
                wallet, "send", "--key-file", alice_key,
                "--to", bob, "--amount", "1.25", "--fee", "0.01",
                "--node", a_address, "--yes",
            )
            ids = re.findall(r"(?m)^transaction_id=([0-9a-fA-F]{64})$", sent)
            if len(ids) != 1:
                raise AssertionError(f"Wallet did not return transaction ID: {sent[-800:]}")
            txid = ids[0]
            # B must learn the pending A->B payment without mining a block.
            await_pending(wallet, b_address, txid)
            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1", "--peer", a_address,
            )
            source3 = state(node, a_data)
            if source3[0] != 3:
                raise AssertionError(f"Wallet payment was not mined: {source3[3]}")
            await_tip(node, b_data, source3)
            for address in (a_address, b_address):
                verification = run(
                    wallet, "verify", "--node", address,
                    "--txid", txid, "--confirmations", "1",
                )
                if "SPV verified:" not in verification:
                    raise AssertionError(
                        f"SPV proof missing on {address}: {verification[-800:]}"
                    )
            balance = run(
                wallet, "balance", "--key-file", bob_key, "--node", b_address,
            )
            if "pending_adjusted_balance=1.25000000" not in balance:
                raise AssertionError(f"Recipient balance wrong after sync: {balance[-800:]}")

            # Exercise the reverse B->A direction. A used to run without a
            # configured peer, so this catches one-way mempool regressions.
            returned = run(
                wallet, "send", "--key-file", bob_key,
                "--to", miner, "--amount", "0.05", "--fee", "0.01",
                "--node", b_address, "--yes",
            )
            returned_ids = re.findall(
                r"(?m)^transaction_id=([0-9a-fA-F]{64})$", returned
            )
            if len(returned_ids) != 1:
                raise AssertionError(
                    f"Reverse payment has no transaction ID: {returned[-800:]}"
                )
            reverse_txid = returned_ids[0]
            # The opposite node must see Pending before either peer mines.
            await_pending(wallet, a_address, reverse_txid)
            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1",
                "--peer", b_address, "--peer", a_address,
            )
            source4 = state(node, a_data)
            if source4[0] != 4:
                raise AssertionError(f"Reverse payment was not mined: {source4[3]}")
            await_tip(node, b_data, source4)
            for address in (a_address, b_address):
                verified = run(
                    wallet, "verify", "--node", address,
                    "--txid", reverse_txid, "--confirmations", "1",
                )
                if "SPV verified:" not in verified:
                    raise AssertionError(
                        f"Reverse payment SPV missing on {address}: {verified[-800:]}"
                    )
            final_balance = run(
                wallet, "balance", "--key-file", bob_key, "--node", b_address,
            )
            if ("pending_adjusted_balance=1.19000000" not in final_balance
                    or "next_nonce=1" not in final_balance):
                raise AssertionError(
                    f"Reverse payment debited Bob incorrectly: {final_balance[-800:]}"
                )

            # Repeat the four rejected wallet cases from the physical
            # two-PC campaign against the actual packaged Windows wallet.
            # Nothing should change in either node or Bob's account.
            wallet_before = account_state(wallet, bob_key, b_address)
            chain_a_before = state(node, a_data)[:3]
            chain_b_before = state(node, b_data)[:3]
            for recipient, amount, error in (
                (miner, "1000", "insufficient pending-adjusted balance"),
                ("ADRESSE_INVALIDE", "0.10",
                 "account id must contain exactly 64 ASCII hexadecimal digits"),
                (bob, "0.10", "sender and recipient must differ"),
                (miner, "0", "payment amount must be positive"),
            ):
                expect_rejected_payment(
                    wallet, bob_key, recipient, amount, b_address, error
                )
            wallet_after = account_state(wallet, bob_key, b_address)
            if wallet_after != wallet_before:
                raise AssertionError(
                    f"Rejected payment mutated wallet state: {wallet_before} -> {wallet_after}"
                )
            if (state(node, a_data)[:3] != chain_a_before
                    or state(node, b_data)[:3] != chain_b_before):
                raise AssertionError("Rejected payment changed persisted chain state")
            stop(b)
            b = None
            stop(a)
            a = None

            persisted_a, persisted_b = state(node, a_data), state(node, b_data)
            if persisted_a[:3] != persisted_b[:3]:
                raise AssertionError(
                    f"Persistent chain divergence: {persisted_a[3]} vs {persisted_b[3]}"
                )
            if persisted_a[0] != 4:
                raise AssertionError(f"Unexpected final height: {persisted_a[3]}")
            print(
                "packaged_windows_two_node_smoke=success "
                f"height={persisted_a[0]} tip={persisted_a[1]} "
                f"known_blocks={persisted_a[2]} "
                f"wallet_payment_spv=success first_recipient_balance=1.25000000 "
                f"bidirectional_mempool_relay=success final_recipient_balance=1.19000000 "
                f"wallet_rejection_no_mutation=success "
                f"initial_sync_seconds={initial:.3f} resumed_sync_seconds={resumed:.3f}"
            )
        finally:
            stop(b)
            stop(a)


if __name__ == "__main__":
    main()
