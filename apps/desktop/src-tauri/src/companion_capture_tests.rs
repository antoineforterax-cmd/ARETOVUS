use super::{capture, CaptureError};
use std::env;
use std::io::{self, Write};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const FIXTURE_MODE: &str = "ARETOVUS_TEST_COMPANION_CAPTURE";
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(3);
const DESCENDANT_LIFETIME: Duration = Duration::from_secs(10);

fn fixture_name() -> String {
    let (_, module) = module_path!().split_once("::").unwrap();
    format!("{module}::capture_fixture")
}

fn test_binary() -> Command {
    Command::new(env::current_exe().unwrap())
}

fn fixture_command(mode: &str) -> Command {
    let mut command = test_binary();
    command
        .args(["--ignored", "--exact", "--nocapture", "--quiet"])
        .arg(fixture_name())
        .env(FIXTURE_MODE, mode);
    command
}

fn listing_command() -> Command {
    let mut command = test_binary();
    command
        .args(["--list", "--format", "terse", "--ignored", "--exact"])
        .arg(fixture_name());
    command
}

fn expected_listing() -> Vec<u8> {
    let output = listing_command().output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(fixture_name().as_str()));
    output.stdout
}

#[test]
#[ignore = "subprocess fixture"]
fn capture_fixture() {
    match env::var(FIXTURE_MODE).as_deref() {
        Ok("spawn") => {
            // The direct child exits immediately, leaving the grandchild's stdout open.
            let _descendant = fixture_command("hold")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            process::exit(0);
        }
        Ok("hold") => thread::sleep(DESCENDANT_LIFETIME),
        Ok("flood") => {
            io::stdout().write_all(&[b'x'; 128 * 1024]).unwrap();
            thread::sleep(DESCENDANT_LIFETIME);
        }
        _ => (),
    }
}

#[test]
fn inherited_stdout_times_out_and_next_capture_succeeds() {
    let expected = expected_listing();
    let start = Instant::now();
    let result = capture(fixture_command("spawn"), None, 64 * 1024, CAPTURE_TIMEOUT);
    let elapsed = start.elapsed();

    assert_eq!(result, Err(CaptureError::Timeout));
    assert!(
        elapsed >= CAPTURE_TIMEOUT,
        "capture timed out after {elapsed:?}"
    );
    assert!(
        elapsed < DESCENDANT_LIFETIME - Duration::from_secs(2),
        "capture waited {elapsed:?} for inherited stdout"
    );
    assert_eq!(
        capture(listing_command(), None, expected.len(), CAPTURE_TIMEOUT).unwrap(),
        expected
    );
}

#[test]
fn capture_accepts_the_limit_and_rejects_one_extra_byte() {
    let expected = expected_listing();
    assert_eq!(
        capture(listing_command(), None, expected.len(), CAPTURE_TIMEOUT).unwrap(),
        expected
    );
    assert_eq!(
        capture(listing_command(), None, expected.len() - 1, CAPTURE_TIMEOUT),
        Err(CaptureError::TooLarge)
    );
}

#[test]
fn capture_enforces_the_limit_while_the_child_is_running() {
    assert_eq!(
        capture(fixture_command("flood"), None, 4096, CAPTURE_TIMEOUT),
        Err(CaptureError::TooLarge)
    );
}
