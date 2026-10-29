# Public NIST ACVP ML-DSA-65 examples

Source: **National Institute of Standards and Technology**, [ACVP-Server](https://github.com/usnistgov/ACVP-Server/tree/975de31eb83d87039ec88934fdc47d8c312b892d), commit `975de31eb83d87039ec88934fdc47d8c312b892d` (release v1.1.0.43). These are sample data for the `FIPS204` revision, not an AURORA certificate or a successful ACVTS submission.

Extracted on 2026-10-27. Changes: selected the ML-DSA-65 groups/cases below, omitted unused fields, and changed JSON representation to simple `name=value` records. Retained field values are unchanged. All seeds and expanded signing keys here are **public test material**, never wallet keys. Never use them for a real account. `NIST-NOTICE.txt` reproduces the complete upstream license notice; the data are not relicensed under the repository's MIT/Apache terms.

| Local fixture | Upstream path under `gen-val/json-files/` | Selection | Original file SHA-256 |
| --- | --- | --- | --- |
| `keygen.txt` | `ML-DSA-keyGen-FIPS204/internalProjection.json` | tgId 2, all tcId 26–50; seed and public key | `e67ee6540d40e11506c3c4e3b1f79fc1cefcd49820db99fc61f87cc8ba463baf` |
| `siggen.txt` | `ML-DSA-sigGen-FIPS204/internalProjection.json` | tgId 3, tcId 39, 41, 44; deterministic external/pure | `72dcaf5f69853ca267ccd16af9cb40949786aca0fcfbf05d1ebeba132b93af22` |
| `sigver.txt` | `ML-DSA-sigVer-FIPS204/internalProjection.json` | tgId 3, all tcId 31–45; external/pure, 3 valid and 12 invalid | `47cdd6314c7f746d02421ffcba89d4dbc7bb875ac49e07a029fdfc26fba55437` |

The signature-generation selection includes a 255-byte context (39), a one-byte message (41), and an empty context with an 8192-byte message (44). Signature-verification examples include altered messages, commitments, hints and cases labelled by NIST as altered `z`. These labels are retained as provenance, not assumed to establish exhaustive boundary coverage. A separate test constructs exact `z` norm boundaries explicitly.

| Local file | SHA-256 after extraction |
| --- | --- |
| `keygen.txt` | `ebfca5fd9375d590f58b6d04f015cbb73e9d59823da64294fcda45d11e541dd5` |
| `siggen.txt` | `8b8964390c8fdf2466f7bb4702c8bc1430086cf878cb8ac6dc046228eb488014` |
| `sigver.txt` | `eb36497f9af719db573f9868ba7e24b0b42044af4aaaf57d654c14c83cabb184` |

To reproduce, download the three original `internalProjection.json` files from the pinned commit to a separate directory as `keygen.json`, `siggen.json`, `sigver.json`, then run:

```sh
python tests/vectors-fips204/extract.py --source-dir /path/to/downloads
```

The extractor refuses different source hashes. Tests read only these checked-in text fixtures; they perform no network requests and add no Rust dependency. The original notice is from the `License` section of the upstream `README.md` at the same commit.
