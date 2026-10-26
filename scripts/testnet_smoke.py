#!/usr/bin/env python3
"""Run a two-process AURORA testnet propagation and convergence smoke test."""
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
    try:
        while time.monotonic() < deadline:
            snapshots = list(data_dir.glob("snapshot-*.aurora"))
            heights = [
                int(match.group(1))
                for snapshot in snapshots
                if (match := re.match(r"snapshot-(\d{20})-", snapshot.name))
            ]
            if heights and max(heights) >= 2:
                saved_height = max(heights)
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

    resumed = run_node(
        binary, "testnet", "mine", "--data-dir", str(data_dir),
        "--miner", miner, "--count", "1",
    )
    heights = [int(value) for value in re.findall(r"height=(\d+)\b", resumed)]
    if not heights or max(heights) <= saved_height:
        raise AssertionError(
            f"node did not resume above the last committed snapshot (height {saved_height}):\n{resumed}"
        )
    return max(heights)


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
    subprocess.run([CARGO, "build", "--locked", "-p", "aurora-node"], cwd=ROOT, check=True)
    subprocess.run([CARGO, "build", "--locked", "-p", "aurora-wallet"], cwd=ROOT, check=True)
    suffix = ".exe" if os.name == "nt" else ""
    binary = ROOT / "target" / "debug" / f"aurora-node{suffix}"
    wallet = ROOT / "target" / "debug" / f"aurora-wallet{suffix}"
    if not binary.is_file() or not wallet.is_file():
        raise FileNotFoundError("built node or wallet executable is missing")

    with tempfile.TemporaryDirectory(prefix="aurora-p2p-smoke-") as temporary:
        base = Path(temporary)
        node_a, node_b = base / "node-a", base / "node-b"
        keyfile = base / "miner.aurkey"
        key_output = run_wallet(wallet, "keygen", "--out", str(keyfile))
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
            for data_dir, address, log in (
                (node_a, addr_a, logs[0]),
                (node_b, addr_b, logs[1]),
            ):
                stream = log.open("w", encoding="utf-8")
                server = subprocess.Popen(
                    [str(binary), "testnet", "serve", "--data-dir", str(data_dir),
                     "--listen", f"{address[0]}:{address[1]}"],
                    cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
                )
                # Keep the parent stream alive until the server exits.
                server._aurora_log_stream = stream  # type: ignore[attr-defined]
                servers.append(server)
                wait_listening(server, address, log)

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

            sent = run_wallet(
                wallet, "send", "--key-file", str(keyfile), "--to", RECIPIENT,
                "--amount", "1.00000000", "--fee", "0.00000003",
                "--node", f"{addr_b[0]}:{addr_b[1]}",
            )
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

            # Restart A after mining so its server loads the persisted height-4 chain.
            for server in servers[:2]:
                server.terminate()
                server.wait(timeout=5)
            stream = logs[0].open("a", encoding="utf-8")
            server_a = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(node_a),
                 "--listen", f"{addr_a[0]}:{addr_a[1]}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
            )
            server_a._aurora_log_stream = stream  # type: ignore[attr-defined]
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
            server_a._aurora_log_stream = stream  # type: ignore[attr-defined]
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
            server_c._aurora_log_stream = stream  # type: ignore[attr-defined]
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
            server_a._aurora_log_stream = stream  # type: ignore[attr-defined]
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

            crash_height = crash_recovery_test(binary, base / "crash-node", miner)
            size_limit_height = write_limit_recovery_test(
                binary, base / "write-limit-node", miner
            )
            print(
                "wallet, SPV, 20-block soak, heavier-branch reorganization, forced-stop "
                "and write-limit recovery passed: "
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
                stream = getattr(server, "_aurora_log_stream", None)
                if stream is not None:
                    stream.close()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"testnet smoke failed: {error}", file=sys.stderr)
        raise
