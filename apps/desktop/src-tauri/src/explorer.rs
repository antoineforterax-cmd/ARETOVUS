//! Local-only, read-only canonical explorer child of the ARETOVUS desktop.
//! Never open sockets, spawn via shell, expose wallet secrets or skip snapshot
//! verification; the explorer executable is a *companion* of this host.
use std::{collections::HashSet, path::Path, process::Command, time::Duration};

use crate::{
    companion_capture::{capture, CaptureError},
    native_error::NativeError::{self, InvalidExplorerResponse},
    BridgeLease,
};
use serde_json::{json, Value};

const MAX_PAGE: usize = 20;
const MAX_OUTPUT: usize = 16 * 1024;
const DEADLINE: Duration = Duration::from_secs(30);
const JS_MAX_SAFE: u64 = 9_007_199_254_740_991;

fn bounded_u64(v: &Value) -> Result<u64, NativeError> {
    v.as_u64()
        .filter(|n| *n <= JS_MAX_SAFE)
        .ok_or(InvalidExplorerResponse)
}

fn hash(v: &Value) -> Result<&str, NativeError> {
    let v = v.as_str().ok_or(InvalidExplorerResponse)?;
    if v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(v)
    } else {
        Err(InvalidExplorerResponse)
    }
}

/// Normalize large native u64 timestamp/nonce to strings before sending to JS
/// so the UI never rounds these values above Number.MAX_SAFE_INTEGER.
fn validate_page(
    page: &Value,
    requested_start: u64,
    requested_limit: usize,
) -> Result<Value, NativeError> {
    if page["schema_version"] != 1
        || page["network"] != "testnet"
        || page["status"] != "verified_local"
        || page["source"] != "aretovus-native-explorer"
        || page["read_only"] != true
        || !(1..=MAX_PAGE).contains(&requested_limit)
    {
        return Err(InvalidExplorerResponse);
    }
    let tip = hash(&page["chain_tip"])?;
    let height = bounded_u64(&page["chain_height"])?;
    let start = bounded_u64(&page["start"])?;
    if start != requested_start {
        return Err(InvalidExplorerResponse);
    }
    let blocks = page["blocks"].as_array().ok_or(InvalidExplorerResponse)?;
    let expected_count = (height + 1)
        .saturating_sub(start)
        .min(requested_limit as u64);
    if blocks.len() as u64 != expected_count {
        return Err(InvalidExplorerResponse);
    }
    let mut normalized = Vec::with_capacity(blocks.len());
    let mut seen_hashes = HashSet::with_capacity(blocks.len());
    let mut previous = None;
    let mut first_parent = None;
    for (index, block) in blocks.iter().enumerate() {
        let number = start
            .checked_add(index as u64)
            .ok_or(InvalidExplorerResponse)?;
        if number > height || bounded_u64(&block["height"])? != number {
            return Err(InvalidExplorerResponse);
        }
        let current_hash = hash(&block["hash"])?;
        let parent = hash(&block["parent"])?;
        if !seen_hashes.insert(current_hash)
            || (current_hash == tip) != (number == height)
            || previous.is_some_and(|h| parent != h)
        {
            return Err(InvalidExplorerResponse);
        }
        if index == 0 && number > 0 {
            first_parent = Some(parent);
        }
        let timestamp = block["timestamp"].as_u64().ok_or(InvalidExplorerResponse)?;
        let nonce = block["nonce"].as_u64().ok_or(InvalidExplorerResponse)?;
        let count = bounded_u64(&block["tx_count"])?;
        let root = hash(&block["merkle_root"])?;
        normalized.push(json!({
            "height": number, "hash": current_hash, "parent": parent,
            "timestamp_decimal": timestamp.to_string(), "nonce_decimal": nonce.to_string(),
            "tx_count": count, "merkle_root": root
        }));
        previous = Some(current_hash);
    }
    // If the page starts above genesis, its first block's parent is earlier.
    if first_parent.is_some_and(|parent| parent == tip || seen_hashes.contains(parent)) {
        return Err(InvalidExplorerResponse);
    }
    let calculated_next = start
        .checked_add(blocks.len() as u64)
        .ok_or(InvalidExplorerResponse)?;
    let next = (calculated_next <= height).then_some(calculated_next);
    if page.get("next_start") != Some(&json!(next)) {
        return Err(InvalidExplorerResponse);
    }
    Ok(json!({
        "schema_version":1,"network":"testnet","status":"verified_local",
        "read_only":true,"source":"aretovus-native-explorer",
        "chain_tip":tip,"chain_height":height,"start":start,
        "next_start":next,"blocks":normalized
    }))
}

fn read_page(data_dir: &str, start: u64, limit: usize) -> Result<Value, NativeError> {
    let _lease = BridgeLease::acquire()?;
    if data_dir.is_empty()
        || data_dir.len() > 1024
        || data_dir.contains('\0')
        || start > JS_MAX_SAFE
        || !(1..=MAX_PAGE).contains(&limit)
    {
        return Err(NativeError::InvalidArguments);
    }
    let directory = Path::new(data_dir)
        .canonicalize()
        .map_err(|_| NativeError::InvalidDataDir)?;
    if !directory.is_dir() {
        return Err(NativeError::InvalidDataDir);
    }
    let executable = std::env::current_exe()
        .map_err(|_| NativeError::ExplorerUnavailable)?
        .with_file_name(if cfg!(windows) {
            "aretovus-explorer.exe"
        } else {
            "aretovus-explorer"
        });
    if !executable.is_file() {
        return Err(NativeError::ExplorerUnavailable);
    }
    let mut command = Command::new(executable);
    command
        .arg("--data-dir")
        .arg(directory)
        .arg("--start")
        .arg(start.to_string())
        .arg("--limit")
        .arg(limit.to_string());
    let bytes = capture(command, None, MAX_OUTPUT, DEADLINE).map_err(|error| match error {
        CaptureError::Unavailable => NativeError::ExplorerUnavailable,
        CaptureError::Timeout => NativeError::ExplorerTimeout,
        CaptureError::InvalidOutput => InvalidExplorerResponse,
        CaptureError::TooLarge | CaptureError::Unsuccessful => NativeError::ExplorerFailed,
    })?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| InvalidExplorerResponse)?;
    validate_page(&value, start, limit)
}

#[tauri::command]
pub(crate) async fn read_canonical_page(
    data_dir: String,
    start: u64,
    limit: usize,
) -> Result<Value, Value> {
    tauri::async_runtime::spawn_blocking(move || read_page(&data_dir, start, limit))
        .await
        .map_err(|_| NativeError::ExplorerUnavailable.into_payload())?
        .map_err(NativeError::into_payload)
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

    fn middle_page() -> Value {
        let mut page = sample();
        let first = "a".repeat(64);
        page["chain_tip"] = json!("c".repeat(64));
        page["chain_height"] = json!(3);
        page["start"] = json!(1);
        page["next_start"] = json!(3);
        page["blocks"][0]["height"] = json!(1);
        page["blocks"][0]["hash"] = json!(first);
        page["blocks"][1]["height"] = json!(2);
        page["blocks"][1]["parent"] = json!(first);
        page["blocks"][1]["hash"] = json!("b".repeat(64));
        page
    }

    #[test]
    fn validates_realistic_page_without_rounding_nonce() {
        let page = validate_page(&sample(), 0, 20).unwrap();
        assert_eq!(page["blocks"][1]["nonce_decimal"], "18446744073709551615");
        assert_eq!(page["blocks"][1]["height"], 1);
        assert!(page["next_start"].is_null());
    }
    #[test]
    fn rejects_short_pages_and_missing_terminal_cursors() {
        let mut shortened = sample();
        shortened["blocks"].as_array_mut().unwrap().truncate(1);
        shortened["next_start"] = json!(1);
        assert_eq!(
            validate_page(&shortened, 0, 20),
            Err(InvalidExplorerResponse)
        );

        let mut missing_cursor = sample();
        missing_cursor.as_object_mut().unwrap().remove("next_start");
        assert_eq!(
            validate_page(&missing_cursor, 0, 20),
            Err(InvalidExplorerResponse)
        );
    }

    #[test]
    fn rejects_duplicate_hashes() {
        let mut duplicate = middle_page();
        duplicate["chain_height"] = json!(4);
        duplicate["next_start"] = json!(4);
        let mut third = duplicate["blocks"][0].clone();
        third["height"] = json!(3);
        third["parent"] = json!("b".repeat(64));
        duplicate["blocks"].as_array_mut().unwrap().push(third);
        assert_eq!(
            validate_page(&duplicate, 1, 3),
            Err(InvalidExplorerResponse)
        );
    }

    #[test]
    fn rejects_tip_below_reported_height() {
        let mut early_tip = middle_page();
        early_tip["blocks"][1]["hash"] = json!("c".repeat(64));
        assert_eq!(
            validate_page(&early_tip, 1, 2),
            Err(InvalidExplorerResponse)
        );
    }

    #[test]
    fn rejects_a_non_genesis_first_parent_at_or_above_its_height() {
        let mut page = middle_page();
        assert!(validate_page(&page, 1, 2).is_ok());
        for forged_parent in ['a', 'b', 'c'] {
            page["blocks"][0]["parent"] = json!(forged_parent.to_string().repeat(64));
            assert_eq!(validate_page(&page, 1, 2), Err(InvalidExplorerResponse));
        }
    }

    #[test]
    fn accepts_an_empty_page_beyond_tip() {
        let mut page = middle_page();
        page["start"] = json!(4);
        page["next_start"] = Value::Null;
        page["blocks"] = json!([]);
        assert!(validate_page(&page, 4, 2).is_ok());
    }

    #[test]
    fn rejects_forged_or_discontinuous_pages() {
        let base = sample();
        let mut bad_pages = vec![];
        for (field, value) in [
            ("source", json!("remote")),
            ("network", json!("mainnet")),
            ("status", json!("unverified")),
            ("read_only", json!(false)),
            ("chain_height", json!(-1)),
            ("chain_tip", json!("<script>")),
            ("start", json!(1)),
            ("next_start", json!(1)),
        ] {
            let mut altered = base.clone();
            altered[field] = value;
            bad_pages.push(altered);
        }
        let mut gap = base.clone();
        gap["blocks"][1]["height"] = json!(3);
        bad_pages.push(gap);
        let mut broken_link = base.clone();
        broken_link["blocks"][1]["parent"] = json!("b".repeat(64));
        bad_pages.push(broken_link);
        let mut forged_tip = base.clone();
        forged_tip["chain_tip"] = json!("b".repeat(64));
        bad_pages.push(forged_tip);
        let mut unsafe_count = base.clone();
        unsafe_count["blocks"][1]["tx_count"] = json!(9_007_199_254_740_992_u64);
        bad_pages.push(unsafe_count);
        for page in bad_pages {
            assert_eq!(validate_page(&page, 0, 20), Err(InvalidExplorerResponse));
        }
        assert!(validate_page(&base, 0, 0).is_err());
        assert!(validate_page(&base, 0, 21).is_err());
    }
}
