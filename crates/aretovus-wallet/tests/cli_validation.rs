//! Regressions for rejecting ambiguous input before keys, files or network are touched.
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aretovus-wallet"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn rejected(args: &[&str], reason: &str) {
    let output = run(args);
    assert!(!output.status.success(), "accepted {args:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(reason),
        "{args:?}: {:?}",
        output
    );
}

#[test]
fn every_command_rejects_unknown_options_before_side_effects() {
    for command in [
        "keygen",
        "backup",
        "restore",
        "recovery-export",
        "recovery-verify",
        "recovery-restore",
        "verify-backup",
        "address",
        "account",
        "balance",
        "limits-create",
        "limits-show",
        "send",
        "sign-offline",
        "broadcast",
        "verify",
        "status",
        "wait",
        "help",
        "-h",
        "--help",
    ] {
        rejected(&[command, "--unknown"], "unsupported option");
    }
}

#[test]
fn send_rejects_ambiguity_before_trying_to_open_a_key() {
    rejected(
        &[
            "send",
            "--key-file",
            "absent.aretkey",
            "--amount",
            "1",
            "--amount",
            "2",
        ],
        "duplicate command option",
    );
    rejected(&["send", "--yes", "--yes"], "duplicate command option");
    rejected(
        &["send", "--key-file", "absent.aretkey", "--amount"],
        "missing option value",
    );
    rejected(&["send", "--key-file", "--to"], "missing option value");
    rejected(&["send", "--yes", "--dry-run"], "cannot be combined");
}

#[test]
fn valid_help_and_unknown_command_have_stable_exit_codes() {
    for command in ["help", "-h", "--help"] {
        assert!(run(&[command]).status.success());
    }
    rejected(&["not-a-command"], "unknown command");
}

#[test]
fn extreme_wait_timeout_fails_before_network_without_a_panic() {
    let txid = "00".repeat(32);
    let output = run(&[
        "wait",
        "--node",
        "127.0.0.1:1",
        "--txid",
        &txid,
        "--timeout-seconds",
        "18446744073709551615",
    ]);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(error.contains("wait timeout is out of range"));
    assert!(!error.contains("panicked"));
}
