//! Versioned, checksummed snapshots for the experimental account chain.
//!
//! Snapshots persist accepted blocks from all branches. Loading replays every block against the
//! caller-provided deterministic genesis state. This is not an optimized database or
//! a backup strategy; incomplete and corrupt snapshots are rejected.
use std::{
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use aurora_consensus::{
    account_block::{AccountBlock, MAX_ACCOUNT_BLOCK_BYTES},
    account_fork_chain::{AccountForkChain, AccountForkChainError},
    difficulty::DifficultyParams,
    reward::RewardParams,
};
use aurora_core::{
    accounts::AccountId,
    codec::{CanonicalDecode, CanonicalEncode},
    hash::sha256_plain,
    state::AccountLedger,
};

const MAGIC: &[u8; 8] = b"AURORSNP";
const FORMAT_VERSION: u32 = 3;
const CHECKSUM_LEN: usize = 32;
const MAX_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;
const MAX_BLOCKS: usize = 100_000;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    InvalidFormat(&'static str),
    ChecksumMismatch,
    Decode(aurora_core::codec::DecodeError),
    Encode(aurora_core::codec::EncodeError),
    Chain(AccountForkChainError),
    NoSnapshot,
}
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for StorageError {}
impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<aurora_core::codec::DecodeError> for StorageError {
    fn from(e: aurora_core::codec::DecodeError) -> Self {
        Self::Decode(e)
    }
}
impl From<aurora_core::codec::EncodeError> for StorageError {
    fn from(e: aurora_core::codec::EncodeError) -> Self {
        Self::Encode(e)
    }
}
impl From<AccountForkChainError> for StorageError {
    fn from(e: AccountForkChainError) -> Self {
        Self::Chain(e)
    }
}

/// Save an immutable snapshot named by height and tip hash. Repeating a save is idempotent.
pub fn save_snapshot(
    dir: impl AsRef<Path>,
    chain: &AccountForkChain,
) -> Result<PathBuf, StorageError> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;
    let bytes = encode_snapshot(chain)?;
    let destination = dir.join(format!(
        "snapshot-{:020}-{}-{}.aurora",
        chain.height(),
        chain.tip_hash(),
        sha256_plain(&bytes)
    ));
    if destination.exists() {
        let existing = read_bounded(&destination)?;
        if existing == bytes {
            return Ok(destination);
        }
        return Err(StorageError::InvalidFormat("snapshot name collision"));
    }
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = dir.join(format!(".snapshot-{}-{sequence}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &destination)?;
        Ok::<_, io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(destination)
}

/// Load and validate every snapshot, then select the longest valid canonical history.
pub fn load_latest_snapshot(
    dir: impl AsRef<Path>,
    initial_state: AccountLedger,
) -> Result<AccountForkChain, StorageError> {
    let mut paths = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "aurora"))
        .collect::<Vec<_>>();
    paths.sort();
    let mut best: Option<AccountForkChain> = None;
    for path in paths {
        let bytes = read_bounded(&path)?;
        let chain = decode_snapshot(&bytes, initial_state.clone())?;
        if best.as_ref().is_none_or(|current| {
            chain.chain_work() > current.chain_work()
                || (chain.chain_work() == current.chain_work()
                    && (chain.tip_hash() < current.tip_hash()
                        || (chain.tip_hash() == current.tip_hash()
                            && chain.known_block_count() > current.known_block_count())))
        }) {
            best = Some(chain);
        }
    }
    best.ok_or(StorageError::NoSnapshot)
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, StorageError> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_SNAPSHOT_BYTES as u64 {
        return Err(StorageError::InvalidFormat("snapshot exceeds byte limit"));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX_SNAPSHOT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(StorageError::InvalidFormat("snapshot exceeds byte limit"));
    }
    Ok(bytes)
}

fn encode_snapshot(chain: &AccountForkChain) -> Result<Vec<u8>, StorageError> {
    let blocks = chain.all_blocks_by_height();
    if blocks.len() > MAX_BLOCKS {
        return Err(StorageError::InvalidFormat("too many blocks"));
    }
    let mut body = Vec::new();
    body.extend_from_slice(MAGIC);
    body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
    body.extend_from_slice(&chain.initial_difficulty_bits().to_be_bytes());
    let params = chain.difficulty_params();
    body.extend_from_slice(&params.target_block_seconds.to_be_bytes());
    body.extend_from_slice(&params.adjustment_interval.to_be_bytes());
    body.extend_from_slice(&params.max_adjustment_bits.to_be_bytes());
    body.extend_from_slice(&params.min_bits.to_be_bytes());
    body.extend_from_slice(&params.max_bits.to_be_bytes());
    let rewards = chain.reward_params();
    body.push(u8::from(rewards.enabled));
    body.extend_from_slice(&rewards.treasury_account.0);
    body.extend_from_slice(&(blocks.len() as u32).to_be_bytes());
    for block in blocks {
        let bytes = block.to_canonical_bytes()?;
        let len = u32::try_from(bytes.len())
            .map_err(|_| StorageError::InvalidFormat("block too large"))?;
        body.extend_from_slice(&len.to_be_bytes());
        body.extend_from_slice(&bytes);
        if body.len() + CHECKSUM_LEN > MAX_SNAPSHOT_BYTES {
            return Err(StorageError::InvalidFormat("snapshot exceeds byte limit"));
        }
    }
    let checksum = sha256_plain(&body);
    body.extend_from_slice(checksum.as_bytes());
    Ok(body)
}

fn decode_snapshot(
    bytes: &[u8],
    initial_state: AccountLedger,
) -> Result<AccountForkChain, StorageError> {
    if bytes.len() < 8 + 4 + 2 + 8 + 8 + 2 + 2 + 2 + 1 + 32 + 4 + CHECKSUM_LEN
        || bytes.len() > MAX_SNAPSHOT_BYTES
    {
        return Err(StorageError::InvalidFormat("invalid snapshot length"));
    }
    let body_len = bytes.len() - CHECKSUM_LEN;
    let (body, checksum) = bytes.split_at(body_len);
    if sha256_plain(body).as_bytes() != checksum {
        return Err(StorageError::ChecksumMismatch);
    }
    if &body[..8] != MAGIC {
        return Err(StorageError::InvalidFormat("bad magic"));
    }
    let version = u32::from_be_bytes(body[8..12].try_into().expect("fixed slice"));
    if version != FORMAT_VERSION {
        return Err(StorageError::InvalidFormat("unsupported snapshot version"));
    }
    let initial_difficulty = u16::from_be_bytes(body[12..14].try_into().expect("fixed slice"));
    let params = DifficultyParams {
        target_block_seconds: u64::from_be_bytes(body[14..22].try_into().expect("fixed slice")),
        adjustment_interval: u64::from_be_bytes(body[22..30].try_into().expect("fixed slice")),
        max_adjustment_bits: u16::from_be_bytes(body[30..32].try_into().expect("fixed slice")),
        min_bits: u16::from_be_bytes(body[32..34].try_into().expect("fixed slice")),
        max_bits: u16::from_be_bytes(body[34..36].try_into().expect("fixed slice")),
    };
    let reward_enabled = match body[36] {
        0 => false,
        1 => true,
        _ => return Err(StorageError::InvalidFormat("invalid reward flag")),
    };
    let treasury_account = AccountId(body[37..69].try_into().expect("fixed account id slice"));
    let rewards = RewardParams {
        enabled: reward_enabled,
        treasury_account,
    };
    let count = u32::from_be_bytes(body[69..73].try_into().expect("fixed slice")) as usize;
    if count == 0 || count > MAX_BLOCKS {
        return Err(StorageError::InvalidFormat("invalid block count"));
    }
    let mut cursor = 73usize;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        let end_len = cursor
            .checked_add(4)
            .ok_or(StorageError::InvalidFormat("length overflow"))?;
        let raw_len = body
            .get(cursor..end_len)
            .ok_or(StorageError::InvalidFormat("truncated block length"))?;
        let len = u32::from_be_bytes(raw_len.try_into().expect("fixed slice")) as usize;
        if len > MAX_ACCOUNT_BLOCK_BYTES {
            return Err(StorageError::InvalidFormat("block exceeds byte limit"));
        }
        cursor = end_len;
        let end = cursor
            .checked_add(len)
            .ok_or(StorageError::InvalidFormat("length overflow"))?;
        let encoded = body
            .get(cursor..end)
            .ok_or(StorageError::InvalidFormat("truncated block"))?;
        blocks.push(AccountBlock::from_canonical_bytes(encoded)?);
        cursor = end;
    }
    if cursor != body.len() {
        return Err(StorageError::InvalidFormat("trailing snapshot bytes"));
    }
    let mut iter = blocks.into_iter();
    let genesis = iter
        .next()
        .ok_or(StorageError::InvalidFormat("missing genesis"))?;
    let mut chain = AccountForkChain::new_with_protocol_params(
        genesis,
        initial_difficulty,
        params,
        rewards,
        initial_state,
    )?;
    for block in iter {
        chain.submit(block)?;
    }
    Ok(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::{hash::Hash256, network::NetworkId};
    use std::sync::atomic::AtomicU64;
    static TEST_ID: AtomicU64 = AtomicU64::new(0);
    fn dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurora-storage-{}-{}",
            std::process::id(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn chain() -> AccountForkChain {
        let genesis =
            AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 1, 0, Vec::new()).unwrap();
        AccountForkChain::new(genesis, 0, AccountLedger::new(NetworkId::Regtest)).unwrap()
    }
    #[test]
    fn snapshot_roundtrip_revalidates_genesis_and_state() {
        let path = dir();
        let mut original = chain();
        let next =
            AccountBlock::build(NetworkId::Regtest, original.tip_hash(), 2, 0, Vec::new()).unwrap();
        original.submit(next).unwrap();
        save_snapshot(&path, &original).unwrap();
        let restored = load_latest_snapshot(&path, AccountLedger::new(NetworkId::Regtest)).unwrap();
        assert_eq!(restored.height(), 1);
        assert_eq!(restored.height(), original.height());
        assert_eq!(restored.tip_hash(), original.tip_hash());
        assert_eq!(restored.state(), original.state());
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn snapshot_roundtrip_keeps_secondary_branches_for_reorganization() {
        let path = dir();
        let mut original = chain();
        let genesis = original.tip_hash();
        let branch_a = AccountBlock::build(NetworkId::Regtest, genesis, 2, 0, Vec::new()).unwrap();
        let branch_b = AccountBlock::build(NetworkId::Regtest, genesis, 3, 0, Vec::new()).unwrap();
        original.submit(branch_a).unwrap();
        original.submit(branch_b).unwrap();
        assert_eq!(original.known_block_count(), 3);
        save_snapshot(&path, &original).unwrap();
        let restored = load_latest_snapshot(&path, AccountLedger::new(NetworkId::Regtest)).unwrap();
        assert_eq!(restored.known_block_count(), 3);
        assert_eq!(restored.tip_hash(), original.tip_hash());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn snapshot_roundtrip_preserves_difficulty_parameters() {
        let path = dir();
        let params = DifficultyParams {
            target_block_seconds: 120,
            adjustment_interval: 60,
            max_adjustment_bits: 2,
            min_bits: 1,
            max_bits: 24,
        };
        let mut genesis =
            AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 1, 0, Vec::new()).unwrap();
        genesis.header.nonce =
            aurora_consensus::proof_of_work::mine_nonce(&genesis.header, 8, 0, 100_000).unwrap();
        let rewards = RewardParams::experimental_testnet(AccountId([7; 32]));
        let original = AccountForkChain::new_with_protocol_params(
            genesis,
            8,
            params,
            rewards,
            AccountLedger::new(NetworkId::Regtest),
        )
        .unwrap();
        save_snapshot(&path, &original).unwrap();
        let restored = load_latest_snapshot(&path, AccountLedger::new(NetworkId::Regtest)).unwrap();
        assert_eq!(restored.initial_difficulty_bits(), 8);
        assert_eq!(restored.difficulty_params(), params);
        assert_eq!(restored.reward_params(), rewards);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn latest_snapshot_uses_cumulative_work_even_when_winning_chain_is_shorter() {
        use aurora_consensus::{
            account_fork_chain::AccountForkChain, difficulty::DifficultyParams,
            proof_of_work::mine_nonce, reward::RewardParams,
        };
        let path = dir();
        let params = DifficultyParams {
            target_block_seconds: 10,
            adjustment_interval: 2,
            max_adjustment_bits: 2,
            min_bits: 1,
            max_bits: 6,
        };
        let mut genesis =
            AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 100, 0, vec![]).unwrap();
        genesis.header.nonce = mine_nonce(&genesis.header, 2, 0, 100_000).unwrap();
        let mut chain = AccountForkChain::new_with_params(
            genesis,
            2,
            params,
            AccountLedger::new(NetworkId::Regtest),
        )
        .unwrap();
        let genesis_hash = chain.tip_hash();
        let mut a1 =
            AccountBlock::build(NetworkId::Regtest, genesis_hash, 1_000, 0, vec![]).unwrap();
        a1.header.nonce = mine_nonce(&a1.header, 2, 0, 100_000).unwrap();
        chain.submit(a1).unwrap();
        let mut a2 =
            AccountBlock::build(NetworkId::Regtest, chain.tip_hash(), 2_000, 0, vec![]).unwrap();
        a2.header.nonce = mine_nonce(&a2.header, 1, 0, 100_000).unwrap();
        chain.submit(a2).unwrap();
        let mut a3 =
            AccountBlock::build(NetworkId::Regtest, chain.tip_hash(), 3_000, 0, vec![]).unwrap();
        a3.header.nonce = mine_nonce(&a3.header, 1, 0, 100_000).unwrap();
        chain.submit(a3).unwrap();
        assert_eq!(chain.height(), 3);
        let old_work = chain.chain_work();
        save_snapshot(&path, &chain).unwrap();

        let mut b1 = AccountBlock::build(NetworkId::Regtest, genesis_hash, 101, 0, vec![]).unwrap();
        b1.header.nonce = mine_nonce(&b1.header, 2, 0, 100_000).unwrap();
        let b1_hash = b1.header.hash().unwrap();
        chain.submit(b1).unwrap();
        let mut b2 = AccountBlock::build(NetworkId::Regtest, b1_hash, 102, 0, vec![]).unwrap();
        b2.header.nonce = mine_nonce(&b2.header, 4, 0, 100_000).unwrap();
        chain.submit(b2).unwrap();
        assert_eq!(chain.height(), 2);
        assert!(chain.chain_work() > old_work);
        save_snapshot(&path, &chain).unwrap();

        let restored = load_latest_snapshot(&path, AccountLedger::new(NetworkId::Regtest)).unwrap();
        assert_eq!(restored.height(), 2);
        assert_eq!(restored.tip_hash(), chain.tip_hash());
        assert!(restored.chain_work() > old_work);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn corrupted_snapshot_is_rejected() {
        let path = dir();
        let file = save_snapshot(&path, &chain()).unwrap();
        let mut bytes = fs::read(&file).unwrap();
        bytes[10] ^= 1;
        fs::write(file, bytes).unwrap();
        assert!(matches!(
            load_latest_snapshot(&path, AccountLedger::new(NetworkId::Regtest)),
            Err(StorageError::ChecksumMismatch)
        ));
        fs::remove_dir_all(path).unwrap();
    }
}
