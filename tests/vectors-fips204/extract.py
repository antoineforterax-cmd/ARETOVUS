"""Extract the pinned public NIST examples; no network access or secret input."""
import argparse
import hashlib
import json
from pathlib import Path

SOURCES = {
    "keygen.json": "e67ee6540d40e11506c3c4e3b1f79fc1cefcd49820db99fc61f87cc8ba463baf",
    "siggen.json": "72dcaf5f69853ca267ccd16af9cb40949786aca0fcfbf05d1ebeba132b93af22",
    "sigver.json": "47cdd6314c7f746d02421ffcba89d4dbc7bb875ac49e07a029fdfc26fba55437",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    args = parser.parse_args()
    destination = Path(__file__).resolve().parent
    for name, digest in SOURCES.items():
        data = (args.source_dir / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != digest:
            raise ValueError(f"source checksum mismatch: {name}")
        source = json.loads(data)
        group_id = 2 if name == "keygen.json" else 3
        group = next(g for g in source["testGroups"] if g["tgId"] == group_id)
        assert group["parameterSet"] == "ML-DSA-65"
        fields = {
            "keygen.json": ("tcId", "seed", "pk"),
            "siggen.json": ("tcId", "sk", "pk", "message", "context", "signature"),
            "sigver.json": ("tcId", "testPassed", "pk", "message", "context", "signature", "reason"),
        }[name]
        cases = [t for t in group["tests"] if name != "siggen.json" or t["tcId"] in (39, 41, 44)]
        lines = ["# Public NIST ACVP examples; see README.md and NIST-NOTICE.txt.",
                 "# Extracted 2026-10-27 without modifying retained field values."]
        for case in cases:
            lines.append("")
            for field in fields:
                value = case[field]
                if isinstance(value, bool):
                    value = str(value).lower()
                lines.append(f"{field}={value}")
        output = destination / name.replace(".json", ".txt")
        output.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
        print(f"{output.name}: {len(cases)} cases, SHA256 {hashlib.sha256(output.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
