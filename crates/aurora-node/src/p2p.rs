//! Small bounded TCP sync protocol for the experimental testnet.
use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    time::Duration,
};

use aurora_consensus::{
    account_block::AccountBlock,
    account_fork_chain::{AccountForkChain, AccountForkChainError},
};
use aurora_core::{
    codec::CanonicalDecode,
    hash::{sha256_plain, Hash256},
};
use aurora_storage::save_snapshot;

const MAX_FRAME: usize = 4 * 1024 * 1024;
const PAGE_BLOCKS: usize = 64;
const MAX_SYNC_BLOCKS: usize = 100_000;
const IO_TIMEOUT: Duration = Duration::from_secs(10);
const HELLO: u8 = 1;
const GET_BLOCKS: u8 = 2;
const HELLO_REPLY: u8 = 0x81;
const BLOCKS_REPLY: u8 = 0x82;
const ERROR_REPLY: u8 = 0xff;

#[derive(Debug)]
pub enum P2pError {
    Io(io::Error),
    Protocol(&'static str),
    Chain(AccountForkChainError),
    Storage(aurora_storage::StorageError),
}
impl std::fmt::Display for P2pError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for P2pError {}
impl From<io::Error> for P2pError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<AccountForkChainError> for P2pError {
    fn from(e: AccountForkChainError) -> Self {
        Self::Chain(e)
    }
}
impl From<aurora_storage::StorageError> for P2pError {
    fn from(e: aurora_storage::StorageError) -> Self {
        Self::Storage(e)
    }
}

/// Fingerprint binds peers to the same genesis, retarget and reward configuration.
pub fn protocol_fingerprint(chain: &AccountForkChain) -> Hash256 {
    let mut bytes = Vec::with_capacity(128);
    let genesis = chain.blocks().next().expect("genesis exists");
    bytes.push(genesis.header.network.to_byte());
    bytes.extend_from_slice(
        genesis
            .header
            .hash()
            .expect("genesis header encodes")
            .as_bytes(),
    );
    bytes.extend_from_slice(&chain.initial_difficulty_bits().to_be_bytes());
    let d = chain.difficulty_params();
    bytes.extend_from_slice(&d.target_block_seconds.to_be_bytes());
    bytes.extend_from_slice(&d.adjustment_interval.to_be_bytes());
    bytes.extend_from_slice(&d.max_adjustment_bits.to_be_bytes());
    bytes.extend_from_slice(&d.min_bits.to_be_bytes());
    bytes.extend_from_slice(&d.max_bits.to_be_bytes());
    let r = chain.reward_params();
    bytes.push(u8::from(r.enabled));
    bytes.extend_from_slice(&r.treasury_account.0);
    sha256_plain(&bytes)
}

fn set_timeouts(stream: &TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    Ok(())
}
fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, P2pError> {
    let mut raw_len = [0; 4];
    stream.read_exact(&mut raw_len)?;
    let len = u32::from_be_bytes(raw_len) as usize;
    if len == 0 || len > MAX_FRAME {
        return Err(P2pError::Protocol("frame length outside limit"));
    }
    let mut frame = vec![0; len];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}
fn write_frame(stream: &mut TcpStream, frame: &[u8]) -> Result<(), P2pError> {
    if frame.is_empty() || frame.len() > MAX_FRAME {
        return Err(P2pError::Protocol("outbound frame exceeds limit"));
    }
    stream.write_all(&(frame.len() as u32).to_be_bytes())?;
    stream.write_all(frame)?;
    stream.flush()?;
    Ok(())
}
fn send_hello(stream: &mut TcpStream, chain: &AccountForkChain) -> Result<(), P2pError> {
    let mut frame = vec![HELLO];
    frame.extend_from_slice(protocol_fingerprint(chain).as_bytes());
    frame.extend_from_slice(&chain.height().to_be_bytes());
    frame.extend_from_slice(chain.tip_hash().as_bytes());
    write_frame(stream, &frame)
}
fn check_hello(frame: &[u8], chain: &AccountForkChain) -> Result<(), P2pError> {
    if frame.len() != 73 || frame[0] != HELLO {
        return Err(P2pError::Protocol("invalid handshake"));
    }
    if &frame[1..33] != protocol_fingerprint(chain).as_bytes() {
        return Err(P2pError::Protocol(
            "peer has different testnet consensus parameters",
        ));
    }
    let genesis = chain
        .blocks()
        .next()
        .expect("genesis exists")
        .header
        .hash()
        .expect("header encodes");
    let peer_height = u64::from_be_bytes(frame[33..41].try_into().expect("fixed slice"));
    let peer_tip = &frame[41..73];
    if peer_height == 0 && peer_tip != genesis.as_bytes() {
        return Err(P2pError::Protocol("peer genesis mismatch"));
    }
    Ok(())
}
fn respond_blocks(
    stream: &mut TcpStream,
    chain: &AccountForkChain,
    start: usize,
) -> Result<(), P2pError> {
    let blocks = chain.all_blocks_by_height();
    if start > blocks.len() {
        return Err(P2pError::Protocol("block cursor out of range"));
    }
    let mut frame = vec![BLOCKS_REPLY];
    frame.extend_from_slice(&(blocks.len() as u32).to_be_bytes());
    let mut count = 0u32;
    let count_offset = frame.len();
    frame.extend_from_slice(&count.to_be_bytes());
    for block in blocks.iter().skip(start).take(PAGE_BLOCKS) {
        let encoded = block
            .to_canonical_bytes()
            .map_err(|_| P2pError::Protocol("block encoding failed"))?;
        if encoded.len() > MAX_FRAME || frame.len().saturating_add(4 + encoded.len()) > MAX_FRAME {
            break;
        }
        frame.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
        frame.extend_from_slice(&encoded);
        count += 1;
    }
    if start < blocks.len() && count == 0 {
        return Err(P2pError::Protocol("block exceeds transport frame limit"));
    }
    frame[count_offset..count_offset + 4].copy_from_slice(&count.to_be_bytes());
    write_frame(stream, &frame)
}
fn read_blocks(frame: &[u8]) -> Result<(usize, Vec<AccountBlock>), P2pError> {
    if frame.len() < 9 || frame[0] != BLOCKS_REPLY {
        return Err(P2pError::Protocol("invalid block page"));
    }
    let total = u32::from_be_bytes(frame[1..5].try_into().expect("fixed slice")) as usize;
    let count = u32::from_be_bytes(frame[5..9].try_into().expect("fixed slice")) as usize;
    if count > PAGE_BLOCKS || total > MAX_SYNC_BLOCKS {
        return Err(P2pError::Protocol("peer inventory exceeds limit"));
    }
    let mut cursor: usize = 9;
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        let len_end = cursor
            .checked_add(4)
            .ok_or(P2pError::Protocol("length overflow"))?;
        let len_bytes = frame
            .get(cursor..len_end)
            .ok_or(P2pError::Protocol("truncated block length"))?;
        let len = u32::from_be_bytes(len_bytes.try_into().expect("fixed slice")) as usize;
        cursor = len_end;
        let end = cursor
            .checked_add(len)
            .ok_or(P2pError::Protocol("length overflow"))?;
        let bytes = frame
            .get(cursor..end)
            .ok_or(P2pError::Protocol("truncated block"))?;
        blocks.push(
            AccountBlock::from_canonical_bytes(bytes)
                .map_err(|_| P2pError::Protocol("invalid canonical block"))?,
        );
        cursor = end;
    }
    if cursor != frame.len() {
        return Err(P2pError::Protocol("trailing block page data"));
    }
    Ok((total, blocks))
}

/// Handle one bounded pull-only peer session; this server never trusts remote state.
pub fn handle_peer(
    mut stream: TcpStream,
    chain: &mut AccountForkChain,
    _data_dir: &Path,
) -> Result<(), P2pError> {
    set_timeouts(&stream)?;
    let hello = read_frame(&mut stream)?;
    if let Err(error) = check_hello(&hello, chain) {
        let _ = write_frame(&mut stream, &[ERROR_REPLY]);
        return Err(error);
    }
    send_hello(&mut stream, chain)?;
    loop {
        let request = match read_frame(&mut stream) {
            Ok(frame) => frame,
            Err(P2pError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        if request.first() != Some(&GET_BLOCKS) || request.len() != 5 {
            return Err(P2pError::Protocol("unknown request"));
        }
        let start = u32::from_be_bytes(request[1..5].try_into().expect("fixed slice")) as usize;
        respond_blocks(&mut stream, chain, start)?;
    }
}

pub fn serve(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
) -> Result<(), P2pError> {
    let listener = TcpListener::bind(address)?;
    println!("testnet peer listening on {}", listener.local_addr()?);
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if let Err(error) = handle_peer(stream, chain, data_dir) {
                    eprintln!("peer rejected: {error}");
                }
            }
            Err(error) => eprintln!("accept failed: {error}"),
        }
    }
    Ok(())
}

/// Download all known blocks in bounded pages; consensus validation decides what is retained.
pub fn sync_from_peer(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
) -> Result<usize, P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    send_hello(&mut stream, chain)?;
    let reply = read_frame(&mut stream)?;
    if reply.first() == Some(&ERROR_REPLY) {
        return Err(P2pError::Protocol("peer rejected network identity"));
    }
    check_hello(&reply, chain)?;
    let mut cursor = 1usize;
    let mut accepted = 0;
    loop {
        let mut request = vec![GET_BLOCKS];
        request.extend_from_slice(
            &(u32::try_from(cursor).map_err(|_| P2pError::Protocol("chain too long"))?)
                .to_be_bytes(),
        );
        write_frame(&mut stream, &request)?;
        let (total, blocks) = read_blocks(&read_frame(&mut stream)?)?;
        if total < cursor || blocks.len() > total.saturating_sub(cursor) {
            return Err(P2pError::Protocol("peer block inventory is inconsistent"));
        }
        if blocks.is_empty() {
            if cursor < total {
                return Err(P2pError::Protocol(
                    "peer returned an empty page before end of inventory",
                ));
            }
            break;
        }
        cursor = cursor
            .checked_add(blocks.len())
            .ok_or(P2pError::Protocol("cursor overflow"))?;
        for block in blocks {
            match chain.submit(block) {
                Ok(_) => accepted += 1,
                Err(AccountForkChainError::DuplicateBlock) => {}
                Err(error) => return Err(P2pError::Chain(error)),
            }
        }
        save_snapshot(data_dir, chain)?;
        if cursor >= total {
            break;
        }
    }
    Ok(accepted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_consensus::{difficulty::DifficultyParams, reward::RewardParams};
    use aurora_core::{hash::Hash256, network::NetworkId, state::AccountLedger};
    use std::{fs, net::Ipv4Addr, sync::mpsc, thread};

    fn chain() -> AccountForkChain {
        let genesis = AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 1, 0, vec![]).unwrap();
        AccountForkChain::new_with_protocol_params(
            genesis,
            0,
            DifficultyParams::fixed(0),
            RewardParams::disabled(),
            AccountLedger::new(NetworkId::Regtest),
        )
        .unwrap()
    }
    #[test]
    fn peer_sync_transfers_and_persists_validated_blocks() {
        let dir = std::env::temp_dir().join(format!("aurora-p2p-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut server_chain = chain();
        let block =
            AccountBlock::build(NetworkId::Regtest, server_chain.tip_hash(), 2, 0, vec![]).unwrap();
        server_chain.submit(block).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel();
        let server_dir = dir.clone();
        let thread = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let result = handle_peer(stream, &mut server_chain, &server_dir);
            tx.send(result.is_ok()).unwrap();
        });
        let mut client = chain();
        assert_eq!(sync_from_peer(address, &mut client, &dir).unwrap(), 1);
        assert_eq!(client.height(), 1);
        assert!(rx.recv_timeout(Duration::from_secs(2)).unwrap());
        thread.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn malformed_or_oversized_peer_inventories_are_rejected() {
        let mut oversized = vec![BLOCKS_REPLY];
        oversized.extend_from_slice(&((MAX_SYNC_BLOCKS as u32) + 1).to_be_bytes());
        oversized.extend_from_slice(&0u32.to_be_bytes());
        assert!(read_blocks(&oversized).is_err());

        let mut trailing = vec![BLOCKS_REPLY];
        trailing.extend_from_slice(&1u32.to_be_bytes());
        trailing.extend_from_slice(&0u32.to_be_bytes());
        trailing.push(0);
        assert!(read_blocks(&trailing).is_err());
    }

    #[test]
    fn handshake_rejects_a_different_consensus_fingerprint() {
        let local = chain();
        let mut peer = local.clone();
        let fingerprint = protocol_fingerprint(&local);
        assert_eq!(fingerprint, protocol_fingerprint(&peer));
        let incompatible_genesis =
            AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 2, 0, vec![]).unwrap();
        peer = AccountForkChain::new(
            incompatible_genesis,
            0,
            AccountLedger::new(NetworkId::Regtest),
        )
        .unwrap();
        let mut hello = vec![HELLO];
        hello.extend_from_slice(protocol_fingerprint(&peer).as_bytes());
        hello.extend_from_slice(&0u64.to_be_bytes());
        hello.extend_from_slice(peer.tip_hash().as_bytes());
        assert!(check_hello(&hello, &local).is_err());
    }
}
