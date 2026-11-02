#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod explorer;
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

const REQUEST: &[u8] = br#"{"schema_version":1,"method":"status"}"#;
const MAX_OUTPUT: u64 = 4096;
const DEADLINE: Duration = Duration::from_secs(30);

// A compromised webview must not spawn unlimited expensive snapshot readers.
static BRIDGE_BUSY: AtomicBool = AtomicBool::new(false);
struct BridgeLease;
impl BridgeLease {
    fn acquire() -> Result<Self, String> {
        BRIDGE_BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "bridge_busy".to_string())?;
        Ok(Self)
    }
}
impl Drop for BridgeLease {
    fn drop(&mut self) {
        BRIDGE_BUSY.store(false, Ordering::Release);
    }
}

/// Validate native IPC data again before letting it cross into the webview.
/// In particular, JSON numbers larger than JS MAX_SAFE_INTEGER are not safe
/// for frontend arithmetic, and the diagnostic work string must not contain
/// HTML/control characters or unbounded payloads.
fn validate_status_response(answer: &serde_json::Value) -> Result<(), String> {
    const JS_MAX_SAFE: u64 = 9_007_199_254_740_991;
    let invalid = || "invalid_response".to_string();
    if answer["schema_version"] != 1
        || answer["status"] != "verified"
        || answer["network"] != "testnet"
        || answer["read_only"] != true
        || answer["source"] != "aretovus-native-stdio"
    {
        return Err(invalid());
    }
    let height = answer["height"].as_u64().ok_or_else(invalid)?;
    let known = answer["known_blocks"].as_u64().ok_or_else(invalid)?;
    if height >= JS_MAX_SAFE || known > JS_MAX_SAFE || known <= height {
        return Err(invalid());
    }
    let tip = answer["tip"].as_str().ok_or_else(invalid)?;
    if tip.len() != 64
        || !tip
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid());
    }
    let work = answer["chain_work_display"].as_str().ok_or_else(invalid)?;
    if work.len() > 255
        || !work.starts_with("ChainWork([")
        || !work.ends_with("])")
        || !work[11..work.len() - 2]
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b',' || b == b' ')
    {
        return Err(invalid());
    }
    Ok(())
}

fn bridge_status(data_dir: &str) -> Result<serde_json::Value, String> {
    let _single_reader = BridgeLease::acquire()?;
    if data_dir.is_empty() || data_dir.len() > 1024 || data_dir.contains('\0') {
        return Err("invalid_data_dir".into());
    }
    let directory = Path::new(data_dir)
        .canonicalize()
        .map_err(|_| "invalid_data_dir".to_string())?;
    if !directory.is_dir() {
        return Err("invalid_data_dir".into());
    }
    // Never take the executable from PATH or a webview-controlled URL.
    // Prototype packaging requires a trusted bridge alongside this executable.
    let current = std::env::current_exe().map_err(|_| "bridge_unavailable".to_string())?;
    let bridge_name = if cfg!(windows) {
        "aretovus-local-bridge.exe"
    } else {
        "aretovus-local-bridge"
    };
    let bridge = current.with_file_name(bridge_name);
    if !bridge.is_file() {
        return Err("bridge_unavailable".into());
    }
    let mut child = Command::new(bridge)
        .arg("--data-dir")
        .arg(directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "bridge_unavailable".to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(REQUEST).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err("bridge_unavailable".into());
        }
    }
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < DEADLINE => thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("bridge_timeout".into());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("bridge_unavailable".into());
            }
        }
    };
    let mut bytes = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        stdout
            .take(MAX_OUTPUT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "invalid_response".to_string())?;
    }
    if !status.success() || bytes.len() as u64 > MAX_OUTPUT {
        return Err("verification_failed".into());
    }
    let answer: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid_response".to_string())?;
    validate_status_response(&answer)?;
    Ok(answer)
}

#[tauri::command]
async fn read_node_status(data_dir: String) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || bridge_status(&data_dir))
        .await
        .map_err(|_| "bridge_unavailable".to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_node_status,
            explorer::read_canonical_page
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ARETOVUS Desktop Lab");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid_status() -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1, "status": "verified", "network": "testnet",
            "read_only": true, "source": "aretovus-native-stdio",
            "height": 1, "known_blocks": 2,
            "tip": "ab".repeat(32), "chain_work_display": "ChainWork([256, 0, 0, 0, 0])"
        })
    }

    #[test]
    fn native_bridge_response_requires_strict_fields() {
        let valid = valid_status();
        assert!(validate_status_response(&valid).is_ok());
        let attacks = [
            ("schema_version", serde_json::json!(2)),
            ("network", serde_json::json!("regtest")),
            ("read_only", serde_json::json!(false)),
            ("source", serde_json::json!("untrusted-web")),
            ("height", serde_json::json!(-1)),
            ("height", serde_json::json!(1.5)),
            ("height", serde_json::json!(9_007_199_254_740_991_u64)),
            ("known_blocks", serde_json::json!(0)),
            ("known_blocks", serde_json::json!(9_007_199_254_740_992_u64)),
            ("tip", serde_json::json!("<script>alert(1)</script>")),
            (
                "chain_work_display",
                serde_json::json!("<svg/onload=alert(1)>"),
            ),
        ];
        for (key, value) in attacks {
            let mut forged = valid.clone();
            forged[key] = value;
            assert_eq!(
                validate_status_response(&forged),
                Err("invalid_response".into()),
                "{key}"
            );
        }
    }

    #[test]
    fn only_one_snapshot_reader_is_allowed() {
        let first = BridgeLease::acquire().unwrap();
        assert!(matches!(BridgeLease::acquire(), Err(ref s) if s == "bridge_busy"));
        drop(first);
        assert!(BridgeLease::acquire().is_ok());
    }
}
