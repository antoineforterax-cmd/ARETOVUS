"""Independent adversarial cases supplied by the integration counter-reviewer."""
import copy
import unittest
from pathlib import Path
import subprocess
import tempfile
from foundation_guard import assert_clean_sources, assert_source_identity, validate_files

VALID = {
    "Cargo.lock": b'version = 4\n[[package]]\nname = "example"\nversion = "0.1.0"\n',
    "rust-toolchain.toml": b'[toolchain]\nchannel = "1.99.1"\ncomponents = ["rustfmt", "clippy"]\n',
}


class BuildGuardTests(unittest.TestCase):
    def test_valid(self):
        validate_files(copy.deepcopy(VALID))

    def test_missing_files(self):
        for path in VALID:
            with self.subTest(path=path):
                files = dict(VALID)
                del files[path]
                with self.assertRaises(ValueError):
                    validate_files(files)

    def test_invalid_lock(self):
        for content in (b"", b"version = 4\n[[package]\n",
                        b"version = 3\n[[package]]\nname='x'\n",
                        b"version = 4\npackage = []\n", b"\xff"):
            with self.subTest(content=content):
                files = dict(VALID, **{"Cargo.lock": content})
                with self.assertRaises(ValueError):
                    validate_files(files)

    def test_wrong_channels(self):
        for channel in (b"stable", b"nightly", b"1.99", b"1.99.0"):
            with self.subTest(channel=channel):
                files = dict(VALID)
                files["rust-toolchain.toml"] = files["rust-toolchain.toml"].replace(b"1.99.1", channel)
                with self.assertRaises(ValueError):
                    validate_files(files)

    def test_missing_components(self):
        for components in (b"[]", b'["rustfmt"]', b'["clippy"]'):
            with self.subTest(components=components):
                files = dict(VALID)
                files["rust-toolchain.toml"] = files["rust-toolchain.toml"].replace(
                    b'["rustfmt", "clippy"]', components)
                with self.assertRaises(ValueError):
                    validate_files(files)

    def test_component_order(self):
        files = dict(VALID)
        files["rust-toolchain.toml"] = files["rust-toolchain.toml"].replace(
            b'["rustfmt", "clippy"]', b'["clippy", "rustfmt"]')
        validate_files(files)

    def test_wrong_toolchain_table_type(self):
        files = dict(VALID, **{"rust-toolchain.toml": b'toolchain = "oops"\n'})
        with self.assertRaises(ValueError):
            validate_files(files)

    def test_float_lock_version(self):
        files = dict(VALID)
        files["Cargo.lock"] = files["Cargo.lock"].replace(b"version = 4", b"version = 4.0", 1)
        with self.assertRaises(ValueError):
            validate_files(files)

    def test_malformed_toolchain(self):
        files = dict(VALID, **{"rust-toolchain.toml": b'[toolchain\nchannel="1.99.1"'})
        with self.assertRaises(ValueError):
            validate_files(files)


class IdentityTests(unittest.TestCase):
    def setUp(self):
        self.expected = {
            "Cargo.toml": ("100644", "1" * 40),
            "crates/aurora-core/Cargo.toml": ("100644", "2" * 40),
            "crates/aurora-core/src/lib.rs": ("100644", "3" * 40),
            "crates/aurora-consensus/src/lib.rs": ("100644", "4" * 40),
            "README.md": ("100644", "5" * 40),
        }

    def test_identical(self):
        assert_source_identity(self.expected, dict(self.expected))

    def test_content_changes(self):
        for path in self.expected:
            if path == "README.md":
                continue
            with self.subTest(path=path):
                actual = dict(self.expected)
                actual[path] = ("100644", "6" * 40)
                with self.assertRaises(ValueError):
                    assert_source_identity(self.expected, actual)

    def test_deletion(self):
        actual = dict(self.expected)
        del actual["crates/aurora-consensus/src/lib.rs"]
        with self.assertRaises(ValueError):
            assert_source_identity(self.expected, actual)

    def test_addition(self):
        actual = dict(self.expected)
        actual["crates/aurora-core/src/extra.rs"] = ("100644", "7" * 40)
        with self.assertRaises(ValueError):
            assert_source_identity(self.expected, actual)

    def test_mode(self):
        actual = dict(self.expected)
        actual["crates/aurora-core/src/lib.rs"] = ("100755", "3" * 40)
        with self.assertRaises(ValueError):
            assert_source_identity(self.expected, actual)

    def test_documentation_and_build_controls(self):
        actual = dict(self.expected)
        actual.update({"README.md": ("100644", "a" * 40),
                       "Cargo.lock": ("100644", "b" * 40),
                       "rust-toolchain.toml": ("100644", "c" * 40)})
        assert_source_identity(self.expected, actual)


class WorkingTreeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.source = self.repo / "crates/example/src/lib.rs"
        self.source.parent.mkdir(parents=True)
        self.source.write_text("// fixture\n")
        (self.repo / "Cargo.toml").write_text("[workspace]\n")
        (self.repo / ".gitignore").write_text("crates/ignored.rs\n")
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "-qm", "test fixture")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.repo, stderr=subprocess.STDOUT)

    def test_clean_checkout(self):
        assert_clean_sources(self.repo)

    def test_modified_tracked_source(self):
        self.source.write_text("// changed\n")
        with self.assertRaises(ValueError):
            assert_clean_sources(self.repo)

    def test_staged_manifest_change(self):
        (self.repo / "Cargo.toml").write_text("[workspace]\nresolver='2'\n")
        self.git("add", "Cargo.toml")
        with self.assertRaises(ValueError):
            assert_clean_sources(self.repo)

    def test_untracked_source(self):
        (self.repo / "crates/extra.rs").write_text("// new\n")
        with self.assertRaises(ValueError):
            assert_clean_sources(self.repo)

    def test_ignored_source(self):
        (self.repo / "crates/ignored.rs").write_text("// ignored but could be compiled\n")
        with self.assertRaises(ValueError):
            assert_clean_sources(self.repo)


if __name__ == "__main__":
    unittest.main()
