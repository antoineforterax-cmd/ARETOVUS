#!/usr/bin/env python3
"""Independent fixed identity / proof-of-work vectors for ARETOVUS testnet."""
import hashlib
import struct


def tagged(name: bytes, data: bytes) -> bytes:
    assert 0 < len(name) < 256
    return hashlib.sha256(bytes((len(name),)) + name + data).digest()


def compute(nonce: int) -> bytes:
    root = tagged(b"ARETOVUS/exp/v1/merkle-empty", b"")
    commitment = tagged(b"ARETOVUS/exp/v0/account-tx-list", root + b"\\x00")
    header = (
        struct.pack(">I", 1)
        + b"\\x02"
        + bytes(32)
        + commitment
        + root
        + struct.pack(">QQ", 1, nonce)
    )
    assert len(header) == 117
    return tagged(b"ARETOVUS/exp/v0/block-header", header)


def main() -> None:
    expected = "002897e98fc5f76ea6b9a9e06eec414c6bcafd7171f73b644971a6b0aefbcfe3"
    assert compute(45).hex() == expected
    assert compute(45)[0] == 0, "new testnet genesis is not valid 8-bit proof of work"
    assert compute(44)[0] != 0, "pinned nonce should be first valid candidate"
    assert all(compute(nonce)[0] != 0 for nonce in range(45))
    assert compute(189).hex() != expected
    print("aretovus_testnet_genesis_and_pow_vectors=ok")


if __name__ == "__main__":
    main()
