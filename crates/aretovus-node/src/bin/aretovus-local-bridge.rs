//! ARETOVUS V2 experimental local read-only IPC: a single bounded stdio request.
//!
//! This process opens no TCP port, never signs and never reads wallet keys.
//! A trusted desktop parent must own/launch this child and enforce a deadline.
use std::{
    env,
    ffi::OsStr,
    io::{self, Read},
    path::PathBuf,
    process,
};

use aretovus_consensus::{
    account_fork_chain::AccountForkChain,
    reward::RewardParams,
    testnet::{testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_INITIAL_BITS, TESTNET_TREASURY},
};
use aretovus_core::{network::NetworkId, state::AccountLedger};
use aretovus_storage::load_latest_snapshot;

const REQUEST_LIMIT: u64 = 256;
const STATUS_REQUEST: &[u8] = br#"{"schema_version":1,"method":"status"}"#;

/// Canonical framing deliberately refuses unknown fields, alternate methods,
/// duplicate keys, nested structures and unbounded payloads. stdin ends at EOF.
fn accept_request<R: Read>(source: R) -> Result<(), &'static str> {
    let mut bytes = Vec::new();
    source
        .take(REQUEST_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "read_failed")?;
    if bytes.len() as u64 > REQUEST_LIMIT {
        return Err("request_too_large");
    }
    let without_newline = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
    let canonical = without_newline
        .strip_suffix(b"\r")
        .unwrap_or(without_newline);
    if canonical != STATUS_REQUEST {
        return Err("invalid_request");
    }
    Ok(())
}

fn get_data_dir() -> Result<PathBuf, &'static str> {
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() != Some(OsStr::new("--data-dir")) {
        return Err("invalid_arguments");
    }
    let data_dir = PathBuf::from(args.next().ok_or("invalid_arguments")?);
    if args.next().is_some() || !data_dir.is_dir() {
        return Err("invalid_data_dir");
    }
    Ok(data_dir)
}

/// Match the real node's identity check; never call save_snapshot or initialize
/// a missing directory. The indexer/GUI must not become consensus authority.
fn verify_native_chain(chain: &AccountForkChain) -> Result<(), &'static str> {
    let expected = testnet_genesis_block()
        .header
        .hash()
        .map_err(|_| "verification_failed")?;
    let stored = chain
        .blocks()
        .next()
        .ok_or("verification_failed")?
        .header
        .hash()
        .map_err(|_| "verification_failed")?;
    if stored != expected
        || chain.initial_difficulty_bits() != TESTNET_INITIAL_BITS
        || chain.difficulty_params() != TESTNET_DIFFICULTY
        || chain.reward_params() != RewardParams::experimental_testnet(TESTNET_TREASURY)
    {
        return Err("verification_failed");
    }
    Ok(())
}

/// JSON values are produced only from validated numeric / hex native Rust
/// types. The debug ChainWork field is DISPLAY ONLY, never a numeric API.
fn render_status(chain: &AccountForkChain) -> String {
    format!(
        "{{\"schema_version\":1,\"network\":\"testnet\",\"status\":\"verified\",\
         \"height\":{},\"tip\":\"{}\",\"known_blocks\":{},\
         \"chain_work_display\":\"{:?}\",\"source\":\"aretovus-native-stdio\",\
         \"read_only\":true}}",
        chain.height(),
        chain.tip_hash(),
        chain.known_block_count(),
        chain.chain_work()
    )
}

fn run() -> Result<String, &'static str> {
    let dir = get_data_dir()?;
    accept_request(io::stdin().lock())?;
    let chain = load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Testnet))
        .map_err(|_| "verification_failed")?;
    verify_native_chain(&chain)?;
    Ok(render_status(&chain))
}

fn main() {
    match run() {
        Ok(response) => println!("{response}"),
        Err(code) => {
            // Never leak user paths, external node stderr, or filesystem errors.
            println!(
                "{{\"schema_version\":1,\"status\":\"error\",\"code\":\"{code}\",\"read_only\":true}}"
            );
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_exact_one_shot_readonly_contract() {
        assert!(accept_request(STATUS_REQUEST).is_ok());
        assert!(accept_request([STATUS_REQUEST, b"\n"].concat().as_slice()).is_ok());
        assert!(accept_request([STATUS_REQUEST, b"\r\n"].concat().as_slice()).is_ok());
        for attack in [
            br#"{"schema_version":1,"method":"mine"}"#.as_slice(),
            br#"{"schema_version":1,"method":"sign"}"#,
            br#"{"schema_version":1,"method":"status","method":"sign"}"#,
            br#"{"schema_version":2,"method":"status"}"#,
            br#"{"schema_version":1, "method":"status"}"#,
            br#"{"schema_version":1,"method":"status","token":"abc"}"#,
            b"",
        ] {
            assert_eq!(accept_request(attack), Err("invalid_request"));
        }
    }

    #[test]
    fn rejects_oversized_and_trailing_frames() {
        assert_eq!(
            accept_request(vec![b'a'; 10_000].as_slice()),
            Err("request_too_large")
        );
        assert_eq!(
            accept_request([STATUS_REQUEST, STATUS_REQUEST].concat().as_slice()),
            Err("invalid_request")
        );
        assert_eq!(
            accept_request([STATUS_REQUEST, b"\nhello"].concat().as_slice()),
            Err("invalid_request")
        );
    }
}
