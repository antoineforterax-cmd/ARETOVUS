#!/usr/bin/env python3
"""Implémentation de référence INDÉPENDANTE (Python, hashlib + struct) des encodages et condensats
du prototype AURORA. Sert à produire les valeurs attendues des tests de vecteurs Rust.

Usage : python3 scripts/reference_vectors.py

Ce script reproduit les mêmes hypothèses que le code Rust (voir docs/DECISION_REGISTER.md, annexe B) :
entiers big-endian à largeur fixe, longueurs sur 4 octets, SHA-256(len(tag) || tag || données).
Si Rust et Python divergent, ce n'est pas "Python qui a raison" : c'est un signal à examiner.
"""
import hashlib
import struct


def u32(x): return struct.pack(">I", x)
def u64(x): return struct.pack(">Q", x)
def sha(b): return hashlib.sha256(b).digest()


def tagged(tag, data):
    assert len(tag) < 256
    return sha(bytes([len(tag)]) + tag + data)


TX = b"AURORA/exp/v0/txid"
HD = b"AURORA/exp/v0/block-header"
TL = b"AURORA/exp/v0/tx-list"


def enc_tx(version, net, inputs, outputs, fee, auth):
    b = u32(version) + bytes([net])
    b += u32(len(inputs)) + b"".join(t + u32(i) for t, i in inputs)
    b += u32(len(outputs)) + b"".join(u64(a) + r for a, r in outputs)
    b += u64(fee) + u32(len(auth)) + auth
    return b


def header(commit, ts, nonce):
    return u32(0) + bytes([1]) + bytes(32) + commit + u64(ts) + u64(nonce)


tx1 = enc_tx(0, 1, [(bytes([0x11]) * 32, 0)], [(5_000_000_000, bytes([0x22]) * 32)], 10_000, bytes([0xAA, 0xBB, 0xCC]))
id1 = tagged(TX, tx1)
tx2 = enc_tx(0, 1, [(id1, 0)], [(4_999_990_000, bytes([0x33]) * 32)], 10_000, b"")
id2 = tagged(TX, tx2)
c0 = tagged(TL, u32(0))
c1 = tagged(TL, u32(1) + id1)
c2 = tagged(TL, u32(2) + id1 + id2)

values = {
    "MAX_UNITS_BE": u64(105_000_000 * 10**8).hex(),
    "SHA_EMPTY": sha(b"").hex(),
    "SHA_ABC": sha(b"abc").hex(),
    "TAG_TXID_EMPTY": tagged(TX, b"").hex(),
    "TAG_HEADER_EMPTY": tagged(HD, b"").hex(),
    "TAG_TXLIST_EMPTY": tagged(TL, b"").hex(),
    "TX1_HEX": tx1.hex(), "TX1_ID": id1.hex(),
    "TX2_HEX": tx2.hex(), "TX2_ID": id2.hex(),
    "COMMIT0": c0.hex(), "COMMIT1": c1.hex(), "COMMIT2": c2.hex(),
}
for name, c in (("H0", c0), ("H1", c1), ("H2", c2)):
    h = header(c, 1_000_000, 42)
    values[name + "_HEX"] = h.hex()
    values[name + "_HASH"] = tagged(HD, h).hex()

for k, v in values.items():
    print(f"{k} = {v}")
