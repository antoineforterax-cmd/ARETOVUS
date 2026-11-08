//! ARETOVUS V2/V5: strictly bounded canonical block page from a verified
//! LOCAL testnet snapshot. No HTTP, P2P sessions, keys or consensus writes.
use std::{env, fmt::Write as FmtWrite, path::PathBuf, process};

use aretovus_consensus::{
    account_fork_chain::AccountForkChain,
    reward::RewardParams,
    testnet::{testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_INITIAL_BITS, TESTNET_TREASURY},
};
use aretovus_core::{
    hash::{Hash256, HASH_LEN},
    network::NetworkId,
    state::AccountLedger,
};
use aretovus_storage::load_latest_snapshot;

const MAX_PAGE: usize = 20;

#[derive(Debug)]
struct Options {
    data_dir: PathBuf,
    selection: PageSelection,
    limit: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum PageSelection {
    Start(u64),
    Hash(Hash256),
}

fn parse_hash(value: &str) -> Result<Hash256, &'static str> {
    if value.len() != 2 * HASH_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("invalid_hash");
    }
    let mut bytes = [0; HASH_LEN];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| "invalid_hash")?;
    }
    Ok(Hash256::from_bytes(bytes))
}

fn parse_options<I: Iterator<Item = std::ffi::OsString>>(args: I) -> Result<Options, &'static str> {
    let mut data_dir = None;
    let mut selection = None;
    let mut limit = None;
    let mut args = args;
    while let Some(flag) = args.next() {
        match flag.to_str() {
            Some("--data-dir") if data_dir.is_none() => {
                data_dir = Some(PathBuf::from(args.next().ok_or("invalid_arguments")?));
            }
            Some("--start") if selection.is_none() => {
                let raw = args.next().ok_or("invalid_arguments")?;
                let value = raw.to_str().ok_or("invalid_start")?;
                if value.is_empty() || !value.bytes().all(|x| x.is_ascii_digit()) {
                    return Err("invalid_start");
                }
                selection = Some(PageSelection::Start(
                    value.parse::<u64>().map_err(|_| "invalid_start")?,
                ));
            }
            Some("--hash") if selection.is_none() => {
                let raw = args.next().ok_or("invalid_arguments")?;
                let value = raw.to_str().ok_or("invalid_hash")?;
                selection = Some(PageSelection::Hash(parse_hash(value)?));
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
        selection: selection.unwrap_or(PageSelection::Start(0)),
        limit: limit.unwrap_or(10),
    })
}

fn require_native_identity(chain: &AccountForkChain) -> Result<(), &'static str> {
    let expected = testnet_genesis_block()
        .header
        .hash()
        .map_err(|_| "verification_failed")?;
    let actual = chain
        .blocks()
        .next()
        .ok_or("verification_failed")?
        .header
        .hash()
        .map_err(|_| "verification_failed")?;
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
    let mut result = format!(
        "{{\"schema_version\":1,\"network\":\"testnet\",\"read_only\":true,\
         \"source\":\"aretovus-native-explorer\",\"status\":\"verified_local\",\
         \"chain_tip\":\"{}\",\"chain_height\":{},\"start\":{},\"blocks\":[",
        chain.tip_hash(),
        chain.height(),
        start
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

fn render_selection(
    chain: &AccountForkChain,
    selection: PageSelection,
    limit: usize,
) -> Result<String, &'static str> {
    if !(1..=MAX_PAGE).contains(&limit) {
        return Err("invalid_limit");
    }
    let start = match selection {
        PageSelection::Start(start) => start,
        PageSelection::Hash(hash) => match chain.canonical_block_height(hash) {
            Some(height) => height - height % limit as u64,
            None => {
                return Ok(format!(
                    "{{\"schema_version\":1,\"network\":\"testnet\",\"read_only\":true,\
                     \"source\":\"aretovus-native-explorer\",\"status\":\"not_found\",\
                     \"chain_tip\":\"{}\",\"chain_height\":{}}}",
                    chain.tip_hash(),
                    chain.height()
                ));
            }
        },
    };
    render_page(chain, start, limit)
}

fn run() -> Result<String, &'static str> {
    let opts = parse_options(env::args_os().skip(1))?;
    let chain = load_latest_snapshot(&opts.data_dir, AccountLedger::new(NetworkId::Testnet))
        .map_err(|_| "verification_failed")?;
    require_native_identity(&chain)?;
    render_selection(&chain, opts.selection, opts.limit)
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
    use aretovus_consensus::{account_block::AccountBlock, proof_of_work::mine_nonce};
    use aretovus_core::accounts::AccountId;
    use std::ffi::OsString;

    fn parse(args: &[&str]) -> Result<(PageSelection, usize), &'static str> {
        let result = parse_options(args.iter().map(|x| OsString::from(*x)))?;
        Ok((result.selection, result.limit))
    }

    #[test]
    fn parser_never_allows_unbounded_pages_or_unknown_options() {
        let cwd = std::env::current_dir().unwrap();
        let folder = cwd.to_string_lossy().into_owned();
        assert_eq!(
            parse(&["--data-dir", &folder, "--start", "23", "--limit", "20"]),
            Ok((PageSelection::Start(23), 20))
        );
        assert_eq!(
            parse(&["--data-dir", &folder]),
            Ok((PageSelection::Start(0), 10))
        );
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

    fn testnet_chain() -> AccountForkChain {
        AccountForkChain::new_with_protocol_params(
            testnet_genesis_block(),
            TESTNET_INITIAL_BITS,
            TESTNET_DIFFICULTY,
            RewardParams::experimental_testnet(TESTNET_TREASURY),
            AccountLedger::new(NetworkId::Testnet),
        )
        .unwrap()
    }

    #[test]
    fn hash_arguments_accept_only_one_canonical_hex_selector() {
        let folder = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let hash = "ab".repeat(HASH_LEN);
        assert_eq!(
            parse(&["--data-dir", &folder, "--hash", &hash]),
            Ok((
                PageSelection::Hash(Hash256::from_bytes([0xab; HASH_LEN])),
                10
            ))
        );
        for invalid in [
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            "g".repeat(64),
            "é".repeat(32),
            format!("+{}", "a".repeat(63)),
        ] {
            assert_eq!(
                parse(&["--data-dir", &folder, "--hash", &invalid]),
                Err("invalid_hash"),
            );
        }
        for options in [
            vec!["--hash"],
            vec!["--hash", &hash, "--hash", &hash],
            vec!["--hash", &hash, "--start", "0"],
            vec!["--start", "0", "--hash", &hash],
        ] {
            let mut args = vec!["--data-dir", &folder];
            args.extend(options);
            assert_eq!(parse(&args), Err("invalid_arguments"));
        }
    }

    #[test]
    fn only_verified_canonical_genesis_is_emitted() {
        let chain = testnet_chain();
        require_native_identity(&chain).unwrap();
        let page = render_selection(&chain, PageSelection::Start(0), 20).unwrap();
        assert!(page.contains("\"chain_height\":0"));
        assert!(page.contains("\"height\":0"));
        assert!(page.contains("\"next_start\":null"));
        assert!(render_selection(&chain, PageSelection::Start(99), 20)
            .unwrap()
            .contains("\"blocks\":[]"));
        assert_eq!(
            render_selection(&chain, PageSelection::Start(0), 21),
            Err("invalid_limit")
        );
    }

    #[test]
    fn hash_lookup_returns_the_aligned_page_or_a_snapshot_bound_miss() {
        let chain = testnet_chain();
        let tip = chain.tip_hash();
        assert_eq!(
            render_selection(&chain, PageSelection::Hash(tip), 10),
            render_selection(&chain, PageSelection::Start(0), 10),
        );
        assert_eq!(
            render_selection(&chain, PageSelection::Hash(Hash256::ZERO), 10).unwrap(),
            format!(
                "{{\"schema_version\":1,\"network\":\"testnet\",\"read_only\":true,\
                 \"source\":\"aretovus-native-explorer\",\"status\":\"not_found\",\
                 \"chain_tip\":\"{tip}\",\"chain_height\":0}}"
            ),
        );
        assert_eq!(
            render_selection(&chain, PageSelection::Hash(tip), 0),
            Err("invalid_limit")
        );
    }

    fn mined_child(parent: Hash256, timestamp: u64) -> AccountBlock {
        let mut block = AccountBlock::build_with_miner(
            NetworkId::Testnet,
            parent,
            timestamp,
            0,
            Some(AccountId([0x42; 32])),
            vec![],
        )
        .unwrap();
        block.header.nonce = mine_nonce(&block.header, TESTNET_INITIAL_BITS, 0, 100_000).unwrap();
        block
    }

    #[test]
    fn hash_lookup_follows_the_selected_branch_after_reorganization() {
        let mut chain = testnet_chain();
        let genesis = chain.tip_hash();
        let genesis_time = chain.tip_timestamp();
        let first = mined_child(genesis, genesis_time + 1);
        let first_hash = first.header.hash().unwrap();
        let second = mined_child(genesis, genesis_time + 2);
        let second_hash = second.header.hash().unwrap();
        chain.submit(first).unwrap();
        chain.submit(second).unwrap();
        let old_tip = chain.tip_hash();
        let side_tip = if old_tip == first_hash {
            second_hash
        } else {
            first_hash
        };
        assert!(chain.contains_block(side_tip));
        let miss = render_selection(&chain, PageSelection::Hash(side_tip), 2).unwrap();
        assert!(miss.contains("\"status\":\"not_found\""));
        assert!(miss.contains(&format!("\"chain_tip\":\"{old_tip}\"")));

        let extension = mined_child(side_tip, genesis_time + 3);
        let new_tip = extension.header.hash().unwrap();
        assert!(chain.submit(extension).unwrap().became_best);
        assert!(chain.contains_block(old_tip));
        let miss = render_selection(&chain, PageSelection::Hash(old_tip), 2).unwrap();
        assert!(miss.contains("\"status\":\"not_found\""));
        assert!(miss.contains(&format!("\"chain_tip\":\"{new_tip}\"")));
        assert_eq!(
            render_selection(&chain, PageSelection::Hash(new_tip), 2),
            render_selection(&chain, PageSelection::Start(2), 2),
        );
        assert_eq!(
            render_selection(&chain, PageSelection::Hash(side_tip), 2),
            render_selection(&chain, PageSelection::Start(0), 2),
        );
    }
}
