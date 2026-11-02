//! ARETOVUS V2/V5: strictly bounded canonical block page from a verified
//! LOCAL testnet snapshot. No HTTP, P2P sessions, keys or consensus writes.
use std::{
    env,
    error::Error,
    ffi::OsStr,
    fmt::Write as FmtWrite,
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

const MAX_PAGE: usize = 20;

#[derive(Debug)]
struct Options {
    data_dir: PathBuf,
    start: u64,
    limit: usize,
}

fn parse_options<I: Iterator<Item = std::ffi::OsString>>(
    args: I,
) -> Result<Options, &'static str> {
    let mut data_dir = None;
    let mut start = None;
    let mut limit = None;
    let mut args = args;
    while let Some(flag) = args.next() {
        match flag.to_str() {
            Some("--data-dir") if data_dir.is_none() => {
                data_dir = Some(PathBuf::from(args.next().ok_or("invalid_arguments")?));
            }
            Some("--start") if start.is_none() => {
                let raw = args.next().ok_or("invalid_arguments")?;
                let value = raw.to_str().ok_or("invalid_start")?;
                if value.is_empty() || !value.bytes().all(|x| x.is_ascii_digit()) {
                    return Err("invalid_start");
                }
                start = Some(value.parse::<u64>().map_err(|_| "invalid_start")?);
            }
            Some("--limit") if limit.is_none() => {
                let raw = args.next().ok_or("invalid_arguments")?;
                let value = raw.to_str().ok_or("invalid_limit")?;
                if value.is_empty() || !value.bytes().all(|x| x.is_ascii_digit()) {
                    return Err("invalid_limit");
                }
                let n = value.parse::<usize>().map_err(|_| "invalid_limit")?;
                if !(1..=MAX_PAGE).contains(&n) {
                    return Err("invalid_limit");
                }
                limit = Some(n);
            }
            _ => return Err("invalid_arguments"),
        }
    }
    let data_dir = data_dir.ok_or("invalid_arguments")?;
    if !data_dir.is_dir() {
        return Err("invalid_data_dir");
    }
    Ok(Options {
        data_dir,
        start: start.unwrap_or(0),
        limit: limit.unwrap_or(10),
    })
}

fn require_native_identity(chain: &AccountForkChain) -> Result<(), &'static str> {
    let expected = testnet_genesis_block()
        .header.hash().map_err(|_| "verification_failed")?;
    let actual = chain.blocks().next()
        .ok_or("verification_failed")?
        .header.hash().map_err(|_| "verification_failed")?;
    if expected != actual
        || chain.initial_difficulty_bits() != TESTNET_INITIAL_BITS
        || chain.difficulty_params() != TESTNET_DIFFICULTY
        || chain.reward_params() != RewardParams::experimental_testnet(TESTNET_TREASURY)
    {
        return Err("verification_failed");
    }
    Ok(())
}

fn render_page(chain: &AccountForkChain, start: u64, limit: usize) -> Result<String, &'static str> {
    if !(1..=MAX_PAGE).contains(&limit) {
        return Err("invalid_limit");
    }
    let mut result = format!(
        "{{\"schema_version\":1,\"network\":\"testnet\",\"read_only\":true,\
         \"source\":\"aretovus-native-explorer\",\"status\":\"verified_local\",\
         \"chain_tip\":\"{}\",\"chain_height\":{},\"start\":{},\"blocks\":[",
        chain.tip_hash(), chain.height(), start
    );
    let mut count = 0u64;
    for block in chain.canonical_block_page(start, limit) {
        let hash = block.header.hash().map_err(|_| "verification_failed")?;
        if count != 0 {
            result.push(',');
        }
        let height = start.checked_add(count).ok_or("invalid_start")?;
        write!(
            result,
            "{{\"height\":{height},\"hash\":\"{hash}\",\"parent\":\"{}\",\
             \"timestamp\":{},\"nonce\":{},\"tx_count\":{},\
             \"merkle_root\":\"{}\"}}",
            block.header.prev_hash,
            block.header.timestamp,
            block.header.nonce,
            block.transactions.len(),
            block.header.tx_merkle_root,
        )
        .map_err(|_| "render_failed")?;
        count += 1;
    }
    result.push_str("],\"next_start\":");
    let next = start.checked_add(count);
    if count == 0 || next.is_none_or(|n| n > chain.height()) {
        result.push_str("null");
    } else if let Some(next) = next {
        write!(result, "{next}").map_err(|_| "render_failed")?;
    }
    result.push('}');
    Ok(result)
}

fn run() -> Result<String, &'static str> {
    let opts = parse_options(env::args_os().skip(1))?;
    let chain = load_latest_snapshot(&opts.data_dir, AccountLedger::new(NetworkId::Testnet))
        .map_err(|_| "verification_failed")?;
    require_native_identity(&chain)?;
    render_page(&chain, opts.start, opts.limit)
}

fn main() {
    match run() {
        Ok(page) => println!("{page}"),
        Err(code) => {
            // Do not print arbitrary paths, snapshot errors, key data or backtraces.
            println!("{{\"schema_version\":1,\"status\":\"error\",\"code\":\"{code}\",\"read_only\":true}}");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn parse(args: &[&str]) -> Result<(u64, usize), &'static str> {
        let result = parse_options(
            args.iter().map(|x| OsString::from(*x))
        )?;
        Ok((result.start, result.limit))
    }

    #[test]
    fn parser_never_allows_unbounded_pages_or_unknown_options() {
        let cwd = std::env::current_dir().unwrap();
        let folder = cwd.to_string_lossy().into_owned();
        assert_eq!(parse(&["--data-dir", &folder, "--start", "23", "--limit", "20"]), Ok((23, 20)));
        assert_eq!(parse(&["--data-dir", &folder]), Ok((0, 10)));
        for args in [
            vec!["--data-dir", &folder, "--limit", "0"],
            vec!["--data-dir", &folder, "--limit", "21"],
            vec!["--data-dir", &folder, "--limit", "-1"],
            vec!["--data-dir", &folder, "--start", "-1"],
            vec!["--data-dir", &folder, "--start", "1.1"],
            vec!["--data-dir", &folder, "--start", "18446744073709551616"],
            vec!["--data-dir", &folder, "--start", "0", "--start", "2"],
            vec!["--data-dir", &folder, "--peer", "127.0.0.1:18444"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn only_verified_canonical_genesis_is_emitted() {
        let chain = AccountForkChain::new_with_protocol_params(
            testnet_genesis_block(),
            TESTNET_INITIAL_BITS, TESTNET_DIFFICULTY,
            RewardParams::experimental_testnet(TESTNET_TREASURY),
            AccountLedger::new(NetworkId::Testnet),
        ).unwrap();
        require_native_identity(&chain).unwrap();
        let page = render_page(&chain, 0, 20).unwrap();
        assert!(page.contains("\"chain_height\":0"));
        assert!(page.contains("\"height\":0"));
        assert!(page.contains("\"next_start\":null"));
        assert!(render_page(&chain, 99, 20).unwrap().contains("\"blocks\":[]"));
        assert_eq!(render_page(&chain, 0, 21), Err("invalid_limit"));
    }
}
