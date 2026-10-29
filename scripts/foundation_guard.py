"""Build/provenance checks for the historical foundation, not consensus validation."""
import json
from pathlib import Path
import subprocess
import tomllib

BASELINE = "eada9effb11d7ef0ad96f082f55feb1558cb90bb"
ALLOWED = {
    "Cargo.lock", "rust-toolchain.toml", ".github/workflows/ci.yml",
    ".github/workflows/foundation-check.yml", "AGENTS.md",
    "docs/planning/IMPLEMENTATION_POLICY.md",
    "docs/consolidation/STATE_2026-10-29.md",
    "docs/consolidation/state-2026-10-29.json",
    "scripts/foundation_guard.py", "scripts/test_foundation_guard.py",
}


def validate_files(files):
    try:
        lock = tomllib.loads(files["Cargo.lock"].decode("utf-8"))
        config = tomllib.loads(files["rust-toolchain.toml"].decode("utf-8"))["toolchain"]
        if lock.get("version") != 4 or not isinstance(lock.get("package"), list) or not lock["package"]:
            raise ValueError("expected a non-empty version 4 Cargo.lock")
        if config.get("channel") != "1.99.1":
            raise ValueError("Rust must be pinned to 1.99.1")
        components = config.get("components", [])
        if not isinstance(components, list) or not {"rustfmt", "clippy"}.issubset(components):
            raise ValueError("rustfmt and clippy are required")
    except (KeyError, UnicodeError, tomllib.TOMLDecodeError, TypeError) as exc:
        raise ValueError("missing or malformed build configuration") from exc


def assert_source_identity(expected, actual):
    def protected(tree):
        return {p: entry for p, entry in tree.items()
                if p == "Cargo.toml" or p.startswith("crates/")}
    if protected(expected) != protected(actual):
        raise ValueError("foundation sources or manifests changed")


def git(*args):
    return subprocess.check_output(["git", *args])


def tree(ref):
    result = {}
    for entry in git("ls-tree", "-r", "-z", ref).split(b"\0"):
        if entry:
            metadata, path = entry.split(b"\t", 1)
            mode, kind, sha = metadata.decode().split()
            result[path.decode()] = (mode, kind, sha)
    return result


def main():
    expected, actual = tree(BASELINE), tree("HEAD")
    assert_source_identity(expected, actual)
    changed = {p for p in expected.keys() | actual.keys()
               if expected.get(p) != actual.get(p)}
    if changed - ALLOWED:
        raise ValueError("unplanned changes: " + repr(sorted(changed - ALLOWED)))
    files = {}
    for path in ("Cargo.lock", "rust-toolchain.toml"):
        if path not in actual or actual[path][0:2] != ("100644", "blob"):
            raise ValueError(path + " must be a tracked regular file")
        files[path] = git("show", "HEAD:" + path)
        if files[path] != Path(path).read_bytes():
            raise ValueError(path + " differs from its committed bytes")
    validate_files(files)
    if "Cargo.lock" in expected:
        raise ValueError("the reproduction baseline unexpectedly contains a lock")
    print(json.dumps({
        "baseline": BASELINE, "candidate": git("rev-parse", "HEAD").decode().strip(),
        "production_sources_identical": True, "changed_files": sorted(changed),
        "rust": "1.99.1", "lock_is_tracked_and_unmodified": True,
    }, indent=2))


if __name__ == "__main__":
    main()
