//! Local-only, read-only canonical explorer child of the ARETOVUS desktop.
//! Never open sockets, spawn via shell, expose wallet secrets or skip snapshot
//! verification; the explorer executable is a *companion* of this host.
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::{json, Value};
use crate::BridgeLease;

const MAX_PAGE: usize = 20;
const MAX_OUTPUT: u64 = 16 * 1024;
const DEADLINE: Duration = Duration::from_secs(30);
const JS_MAX_SAFE: u64 = 9_007_199_254_740_991;

fn bad() -> String { "invalid_explorer_response".into() }

fn bounded_u64(v: &Value) -> Result<u64, String> {
    v.as_u64().filter(|n| *n <= JS_MAX_SAFE).ok_or_else(bad)
}

fn hash(v: &Value) -> Result<&str, String> {
    let v = v.as_str().ok_or_else(bad)?;
    if v.len() == 64 && v.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        Ok(v)
    } else {
        Err(bad())
    }
}

/// Normalize large native u64 timestamp/nonce to strings before sending to JS
/// so the UI never rounds these values above Number.MAX_SAFE_INTEGER.
fn validate_page(page: &Value, requested_start: u64, requested_limit: usize) -> Result<Value, String> {
    if page["schema_version"] != 1 || page["network"] != "testnet"
        || page["status"] != "verified_local"
        || page["source"] != "aretovus-native-explorer"
        || page["read_only"] != true || !(1..=MAX_PAGE).contains(&requested_limit)
    {
        return Err(bad());
    }
    let tip = hash(&page["chain_tip"])?;
    let height = bounded_u64(&page["chain_height"])?;
    let start = bounded_u64(&page["start"])?;
    if start != requested_start { return Err(bad()); }
    let blocks = page["blocks"].as_array().ok_or_else(bad)?;
    if blocks.len() > requested_limit { return Err(bad()); }
    if blocks.is_empty() && start <= height { return Err(bad()); }
    let mut normalized = Vec::with_capacity(blocks.len());
    let mut previous: Option<String> = None;
    for (index, block) in blocks.iter().enumerate() {
        let number = start.checked_add(index as u64).ok_or_else(bad)?;
        if number > height || bounded_u64(&block["height"])? != number {
            return Err(bad());
        }
        let current_hash = hash(&block["hash"])?;
        let parent = hash(&block["parent"])?;
        if previous.as_deref().is_some_and(|h| parent != h) { return Err(bad()); }
        let timestamp = block["timestamp"].as_u64().ok_or_else(bad)?;
        let nonce = block["nonce"].as_u64().ok_or_else(bad)?;
        let count = bounded_u64(&block["tx_count"])?;
        let root = hash(&block["merkle_root"])?;
        normalized.push(json!({
            "height": number, "hash": current_hash, "parent": parent,
            "timestamp_decimal": timestamp.to_string(), "nonce_decimal": nonce.to_string(),
            "tx_count": count, "merkle_root": root
        }));
        previous = Some(current_hash.to_owned());
    }
    if start <= height && !blocks.is_empty() && start + (blocks.len() as u64) - 1 == height
        && previous.as_deref() != Some(tip)
    {
        return Err(bad());
    }
    let calculated_next = start.checked_add(blocks.len() as u64).ok_or_else(bad)?;
    let next = match page["next_start"].as_u64() {
        Some(next) if next == calculated_next && next <= height && !blocks.is_empty() => Some(next),
        Some(_) => return Err(bad()),
        None if page["next_start"].is_null()
            && (blocks.is_empty() || calculated_next > height) => None,
        None => return Err(bad()),
    };
    Ok(json!({
        "schema_version":1,"network":"testnet","status":"verified_local",
        "read_only":true,"source":"aretovus-native-explorer",
        "chain_tip":tip,"chain_height":height,"start":start,
        "next_start":next,"blocks":normalized
    }))
}

fn read_page(data_dir: &str, start: u64, limit: usize) -> Result<Value, String> {
    let _lease = BridgeLease::acquire()?;
    if data_dir.is_empty() || data_dir.len() > 1024 || data_dir.contains('\0')
        || start > JS_MAX_SAFE || !(1..=MAX_PAGE).contains(&limit) {
        return Err("invalid_arguments".into());
    }
    let directory = Path::new(data_dir)
        .canonicalize().map_err(|_| "invalid_data_dir".to_string())?;
    if !directory.is_dir() { return Err("invalid_data_dir".into()); }
    let executable = std::env::current_exe().map_err(|_| "explorer_unavailable".to_string())?
        .with_file_name(if cfg!(windows) { "aretovus-explorer.exe" } else { "aretovus-explorer" });
    if !executable.is_file() { return Err("explorer_unavailable".into()); }
    let mut child = Command::new(executable)
        .arg("--data-dir").arg(directory)
        .arg("--start").arg(start.to_string())
        .arg("--limit").arg(limit.to_string())
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn().map_err(|_| "explorer_unavailable".to_string())?;
    // A page of 20 blocks can exceed the OS pipe buffer. Drain concurrently
    // instead of deadlocking by waiting for exit before reading stdout.
    let stdout = child.stdout.take().ok_or_else(bad)?;
    let reader = thread::spawn(move || {
        let mut out = Vec::new();
        stdout.take(MAX_OUTPUT + 1).read_to_end(&mut out).map(|_| out)
    });
    let began = Instant::now();
    let exit = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if began.elapsed() < DEADLINE => thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("explorer_timeout".into());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("explorer_unavailable".into());
            }
        }
    };
    let bytes = reader.join().map_err(|_| bad())?.map_err(|_| bad())?;
    if !exit.success() || bytes.len() as u64 > MAX_OUTPUT {
        return Err("explorer_failed".into());
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| bad())?;
    validate_page(&value, start, limit)
}

#[tauri::command]
pub(crate) async fn read_canonical_page(
    data_dir: String, start: u64, limit: usize
) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || read_page(&data_dir, start, limit))
        .await.map_err(|_| "explorer_unavailable".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Value {
        let zero = "0".repeat(64);
        let hash = "a".repeat(64);
        json!({
            "schema_version":1,"network":"testnet","status":"verified_local",
            "read_only":true,"source":"aretovus-native-explorer",
            "chain_tip":hash,"chain_height":1,"start":0,"next_start":null,
            "blocks":[
                {"height":0,"hash":zero,"parent":zero,"timestamp":1,"nonce":0,
                 "tx_count":0,"merkle_root":zero},
                {"height":1,"hash":hash,"parent":zero,"timestamp":2,
                 "nonce":18446744073709551615_u64,"tx_count":0,"merkle_root":zero}
            ]
        })
    }
    #[test]
    fn validates_realistic_page_without_rounding_nonce() {
        let page = validate_page(&sample(),0,20).unwrap();
        assert_eq!(page["blocks"][1]["nonce_decimal"], "18446744073709551615");
        assert_eq!(page["blocks"][1]["height"], 1);
        assert!(page["next_start"].is_null());
    }
    #[test]
    fn rejects_forged_or_discontinuous_pages() {
        let base = sample();
        let mut bad_pages = vec![];
        for (field,value) in [
            ("source", json!("remote")), ("network", json!("mainnet")),
            ("status", json!("unverified")),("read_only",json!(false)),
            ("chain_height",json!(-1)),("chain_tip",json!("<script>")),
            ("start",json!(1)),("next_start",json!(1))
        ] {
            let mut altered = base.clone();
            altered[field] = value; bad_pages.push(altered);
        }
        let mut gap = base.clone(); gap["blocks"][1]["height"] = json!(3); bad_pages.push(gap);
        let mut broken_link = base.clone();
        broken_link["blocks"][1]["parent"] = json!("b".repeat(64));
        bad_pages.push(broken_link);
        let mut forged_tip = base.clone();
        forged_tip["chain_tip"] = json!("b".repeat(64)); bad_pages.push(forged_tip);
        let mut unsafe_count = base.clone();
        unsafe_count["blocks"][1]["tx_count"] = json!(9_007_199_254_740_992_u64);
        bad_pages.push(unsafe_count);
        for page in bad_pages {
            assert_eq!(validate_page(&page,0,20),Err("invalid_explorer_response".into()));
        }
        assert!(validate_page(&base,0,0).is_err());
        assert!(validate_page(&base,0,21).is_err());
    }
}
