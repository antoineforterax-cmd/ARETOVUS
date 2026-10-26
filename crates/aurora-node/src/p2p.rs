//! Small bounded TCP sync protocol for the experimental testnet.
use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use aurora_consensus::{
    account_block::AccountBlock,
    account_fork_chain::{AccountForkChain, AccountForkChainError},
};
use aurora_core::{
    codec::{CanonicalDecode, CanonicalEncode},
    hash::{sha256_plain, Hash256},
};
use aurora_storage::save_snapshot;

const MAX_FRAME: usize = 4 * 1024 * 1024;
const PAGE_BLOCKS: usize = 64;
const MAX_SYNC_BLOCKS: usize = 100_000;
pub const MAX_DISCOVERED_PEERS: usize = 64;
const MAX_ADVERTISED_PEERS: usize = 32;
/// Hard aggregate cap per peer session, independent of per-frame limits.
const MAX_SESSION_BYTES: usize = 256 * 1024 * 1024;
const MAX_SESSION_REQUESTS: usize = 16_384;
const MAX_SESSION_DURATION: Duration = Duration::from_secs(15 * 60);
const IO_TIMEOUT: Duration = Duration::from_secs(10);
const PEER_RETRY_ATTEMPTS: usize = 3;
const PEER_RETRY_BASE_DELAY: Duration = Duration::from_millis(100);
const HELLO: u8 = 1;
const GET_BLOCKS: u8 = 2;
const SUBMIT_BLOCK: u8 = 3;
const BLOCK_ACCEPTED: u8 = 0x83;
const BLOCKS_REPLY: u8 = 0x82;
const ERROR_REPLY: u8 = 0xff;
const GET_PEERS: u8 = 4;
const PEERS_REPLY: u8 = 0x84;

#[derive(Debug)]
pub enum P2pError {
    Io(io::Error),
    Protocol(&'static str),
    Chain(AccountForkChainError),
    Storage(aurora_storage::StorageError),
    ChainLockPoisoned,
}
impl std::fmt::Display for P2pError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "peer I/O failed: {error}"),
            Self::Protocol(message) => write!(f, "peer protocol error: {message}"),
            Self::Chain(error) => write!(f, "peer block rejected by consensus: {error}"),
            Self::Storage(error) => write!(f, "cannot persist peer sync: {error}"),
            Self::ChainLockPoisoned => write!(f, "peer chain state lock poisoned"),
        }
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

fn add_session_bytes(total: &mut usize, bytes: usize) -> Result<(), P2pError> {
    *total = total
        .checked_add(bytes)
        .ok_or(P2pError::Protocol("session byte counter overflow"))?;
    if *total > MAX_SESSION_BYTES {
        return Err(P2pError::Protocol("peer session byte budget exceeded"));
    }
    Ok(())
}

fn add_session_request(count: &mut usize) -> Result<(), P2pError> {
    *count = count
        .checked_add(1)
        .ok_or(P2pError::Protocol("session request counter overflow"))?;
    if *count > MAX_SESSION_REQUESTS {
        return Err(P2pError::Protocol("peer session request limit exceeded"));
    }
    Ok(())
}

fn encode_peer_list(peers: &[SocketAddr]) -> Result<Vec<u8>, P2pError> {
    if peers.len() > MAX_ADVERTISED_PEERS {
        return Err(P2pError::Protocol("advertised peer list exceeds limit"));
    }
    let count = u16::try_from(peers.len())
        .map_err(|_| P2pError::Protocol("advertised peer count overflow"))?;
    let mut frame = vec![PEERS_REPLY];
    frame.extend_from_slice(&count.to_be_bytes());
    for peer in peers {
        match peer {
            SocketAddr::V4(address) => {
                frame.push(4);
                frame.extend_from_slice(&address.ip().octets());
                frame.extend_from_slice(&address.port().to_be_bytes());
            }
            SocketAddr::V6(address) => {
                if address.scope_id() != 0 {
                    return Err(P2pError::Protocol("scoped IPv6 peers are not supported"));
                }
                frame.push(6);
                for segment in address.ip().segments() {
                    frame.extend_from_slice(&segment.to_be_bytes());
                }
                frame.extend_from_slice(&address.port().to_be_bytes());
            }
        }
    }
    Ok(frame)
}

fn decode_peer_list(frame: &[u8]) -> Result<Vec<SocketAddr>, P2pError> {
    if frame.len() < 3 || frame[0] != PEERS_REPLY {
        return Err(P2pError::Protocol("invalid peer list"));
    }
    let count = u16::from_be_bytes(frame[1..3].try_into().expect("fixed slice")) as usize;
    if count > MAX_ADVERTISED_PEERS {
        return Err(P2pError::Protocol("advertised peer list exceeds limit"));
    }
    let mut cursor = 3usize;
    let mut peers = Vec::with_capacity(count);
    for _ in 0..count {
        let family = *frame
            .get(cursor)
            .ok_or(P2pError::Protocol("truncated peer address"))?;
        cursor += 1;
        let address_len = match family {
            4 => 4,
            6 => 16,
            _ => return Err(P2pError::Protocol("unknown peer address family")),
        };
        let address_end = cursor
            .checked_add(address_len)
            .ok_or(P2pError::Protocol("peer address length overflow"))?;
        let address = frame
            .get(cursor..address_end)
            .ok_or(P2pError::Protocol("truncated peer address"))?;
        cursor = address_end;
        let port_end = cursor
            .checked_add(2)
            .ok_or(P2pError::Protocol("peer port length overflow"))?;
        let port_bytes = frame
            .get(cursor..port_end)
            .ok_or(P2pError::Protocol("truncated peer port"))?;
        let port = u16::from_be_bytes(port_bytes.try_into().expect("fixed slice"));
        cursor = port_end;
        let ip = if family == 4 {
            std::net::IpAddr::V4(std::net::Ipv4Addr::from(
                <[u8; 4]>::try_from(address).expect("fixed IPv4 address"),
            ))
        } else {
            std::net::IpAddr::V6(std::net::Ipv6Addr::from(
                <[u8; 16]>::try_from(address).expect("fixed IPv6 address"),
            ))
        };
        if port == 0
            || ip.is_unspecified()
            || ip.is_multicast()
            || matches!(ip, std::net::IpAddr::V6(ipv6) if ipv6.is_unicast_link_local())
        {
            return Err(P2pError::Protocol(
                "advertised peer address is not routable",
            ));
        }
        let peer = SocketAddr::new(ip, port);
        if !peers.contains(&peer) {
            peers.push(peer);
        }
    }
    if cursor != frame.len() {
        return Err(P2pError::Protocol("trailing peer list data"));
    }
    Ok(peers)
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
fn build_blocks_page(chain: &AccountForkChain, start: usize) -> Result<Vec<u8>, P2pError> {
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
    Ok(frame)
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
fn handle_peer_shared(
    mut stream: TcpStream,
    chain: &Mutex<AccountForkChain>,
    data_dir: &Path,
    advertised_peers: &[SocketAddr],
) -> Result<(), P2pError> {
    set_timeouts(&stream)?;
    let session_started = Instant::now();
    let mut session_bytes = 0usize;
    let mut session_requests = 0usize;
    let hello = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, hello.len() + 4)?;
    {
        let chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
        if let Err(error) = check_hello(&hello, &chain) {
            let _ = write_frame(&mut stream, &[ERROR_REPLY]);
            return Err(error);
        }
        if 77 > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
            return Err(P2pError::Protocol("peer session byte budget exceeded"));
        }
        send_hello(&mut stream, &chain)?;
    }
    add_session_bytes(&mut session_bytes, 77)?;
    loop {
        if session_started.elapsed() > MAX_SESSION_DURATION {
            return Err(P2pError::Protocol("peer session duration exceeded"));
        }
        let request = match read_frame(&mut stream) {
            Ok(frame) => frame,
            Err(P2pError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        add_session_request(&mut session_requests)?;
        add_session_bytes(&mut session_bytes, request.len() + 4)?;
        match request.first() {
            Some(&GET_BLOCKS) if request.len() == 5 => {
                let start =
                    u32::from_be_bytes(request[1..5].try_into().expect("fixed slice")) as usize;
                let frame = {
                    let chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                    build_blocks_page(&chain, start)?
                };
                let sent = 4 + frame.len();
                if sent > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &frame)?;
                add_session_bytes(&mut session_bytes, sent)?;
            }
            Some(&GET_PEERS) if request.len() == 1 => {
                let frame = encode_peer_list(advertised_peers)?;
                let sent = frame.len() + 4;
                if sent > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &frame)?;
                add_session_bytes(&mut session_bytes, sent)?;
            }
            Some(&SUBMIT_BLOCK) if request.len() > 1 => {
                if 45 > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                let block = AccountBlock::from_canonical_bytes(&request[1..])
                    .map_err(|_| P2pError::Protocol("invalid canonical block"))?;
                let mut chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                match chain.submit(block) {
                    Ok(_) => {
                        save_snapshot(data_dir, &chain)?;
                    }
                    Err(AccountForkChainError::DuplicateBlock) => {}
                    Err(error) => return Err(P2pError::Chain(error)),
                }
                let mut reply = vec![BLOCK_ACCEPTED];
                reply.extend_from_slice(&chain.height().to_be_bytes());
                reply.extend_from_slice(chain.tip_hash().as_bytes());
                drop(chain);
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, 45)?;
            }
            _ => return Err(P2pError::Protocol("unknown or malformed request")),
        }
    }
}

pub fn serve(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
    advertised_peers: &[SocketAddr],
) -> Result<(), P2pError> {
    const PEER_WORKERS: usize = 8;
    let listener = TcpListener::bind(address)?;
    println!("testnet peer listening on {}", listener.local_addr()?);
    let chain = Arc::new(Mutex::new(chain.clone()));
    let data_dir = Arc::new(data_dir.to_path_buf());
    let mut peers = advertised_peers.to_vec();
    peers.sort_unstable();
    peers.dedup();
    peers.truncate(MAX_ADVERTISED_PEERS);
    let advertised_peers = Arc::new(peers);
    let mut workers = Vec::with_capacity(PEER_WORKERS);
    for worker_id in 0..PEER_WORKERS {
        let listener = listener.try_clone()?;
        let chain = Arc::clone(&chain);
        let data_dir = Arc::clone(&data_dir);
        let advertised_peers = Arc::clone(&advertised_peers);
        workers.push(
            thread::Builder::new()
                .name(format!("aurora-peer-{worker_id}"))
                .spawn(move || loop {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            if let Err(error) =
                                handle_peer_shared(stream, &chain, &data_dir, &advertised_peers)
                            {
                                eprintln!("peer rejected: {error}");
                            }
                        }
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(error) => {
                            eprintln!("peer accept failed: {error}");
                            break;
                        }
                    }
                })?,
        );
    }
    for worker in workers {
        worker
            .join()
            .map_err(|_| P2pError::Protocol("peer worker panicked"))?;
    }
    Ok(())
}

fn retryable_peer_error(error: &P2pError) -> bool {
    matches!(
        error,
        P2pError::Io(io_error)
            if matches!(
                io_error.kind(),
                io::ErrorKind::TimedOut
                    | io::ErrorKind::WouldBlock
                    | io::ErrorKind::Interrupted
                    | io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::NotConnected
                    | io::ErrorKind::BrokenPipe
            )
    )
}

fn retry_peer_operation<T>(
    mut operation: impl FnMut() -> Result<T, P2pError>,
) -> Result<T, P2pError> {
    let mut delay = PEER_RETRY_BASE_DELAY;
    for attempt in 0..PEER_RETRY_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if retryable_peer_error(&error) && attempt + 1 < PEER_RETRY_ATTEMPTS => {
                thread::sleep(delay);
                delay = delay.saturating_mul(2);
            }
            Err(error) => return Err(error),
        }
    }
    Err(P2pError::Protocol("peer retry attempts exhausted"))
}

/// Send one locally validated block to a configured peer for independent consensus validation.
pub fn propagate_block(
    address: SocketAddr,
    chain: &AccountForkChain,
    block: &AccountBlock,
) -> Result<(), P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    send_hello(&mut stream, chain)?;
    check_hello(&read_frame(&mut stream)?, chain)?;
    let encoded = block
        .to_canonical_bytes()
        .map_err(|_| P2pError::Protocol("block encoding failed"))?;
    let mut request = vec![SUBMIT_BLOCK];
    request.extend_from_slice(&encoded);
    write_frame(&mut stream, &request)?;
    let reply = read_frame(&mut stream)?;
    if reply.len() != 41 || reply[0] != BLOCK_ACCEPTED {
        return Err(P2pError::Protocol("peer did not acknowledge block"));
    }
    Ok(())
}

/// Retry transient connection failures a small fixed number of times.
pub fn propagate_block_with_retry(
    address: SocketAddr,
    chain: &AccountForkChain,
    block: &AccountBlock,
) -> Result<(), P2pError> {
    retry_peer_operation(|| propagate_block(address, chain, block))
}

/// Download all known blocks in bounded pages; consensus validation decides what is retained.
pub fn sync_from_peer(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
) -> Result<usize, P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    let session_started = Instant::now();
    let mut session_bytes = 77usize;
    let mut session_requests = 0usize;
    send_hello(&mut stream, chain)?;
    let reply = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, reply.len() + 4)?;
    if reply.first() == Some(&ERROR_REPLY) {
        return Err(P2pError::Protocol("peer rejected network identity"));
    }
    check_hello(&reply, chain)?;
    let mut cursor = 1usize;
    let mut accepted = 0;
    loop {
        if session_started.elapsed() > MAX_SESSION_DURATION {
            return Err(P2pError::Protocol("peer session duration exceeded"));
        }
        add_session_request(&mut session_requests)?;
        add_session_bytes(&mut session_bytes, 9)?;
        let mut request = vec![GET_BLOCKS];
        request.extend_from_slice(
            &(u32::try_from(cursor).map_err(|_| P2pError::Protocol("chain too long"))?)
                .to_be_bytes(),
        );
        write_frame(&mut stream, &request)?;
        let page = read_frame(&mut stream)?;
        add_session_bytes(&mut session_bytes, page.len() + 4)?;
        let (total, blocks) = read_blocks(&page)?;
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

/// Request one bounded list of bootstrap peers from a fingerprint-compatible peer.
pub fn discover_peers(
    address: SocketAddr,
    chain: &AccountForkChain,
) -> Result<Vec<SocketAddr>, P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    let mut session_bytes = 77usize;
    send_hello(&mut stream, chain)?;
    let hello = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, hello.len() + 4)?;
    if hello.first() == Some(&ERROR_REPLY) {
        return Err(P2pError::Protocol("peer rejected network identity"));
    }
    check_hello(&hello, chain)?;
    write_frame(&mut stream, &[GET_PEERS])?;
    add_session_bytes(&mut session_bytes, 5)?;
    let frame = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, frame.len() + 4)?;
    decode_peer_list(&frame)
}

pub fn discover_peers_with_retry(
    address: SocketAddr,
    chain: &AccountForkChain,
) -> Result<Vec<SocketAddr>, P2pError> {
    retry_peer_operation(|| discover_peers(address, chain))
}

/// Retry transient connection failures; malformed messages and consensus errors fail immediately.
pub fn sync_from_peer_with_retry(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
) -> Result<usize, P2pError> {
    retry_peer_operation(|| sync_from_peer(address, chain, data_dir))
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
            let shared_chain = Mutex::new(server_chain);
            let result = handle_peer_shared(stream, &shared_chain, &server_dir, &[]);
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
    fn concurrent_peer_sessions_share_chain_without_serializing_network_io() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-concurrent-{}", std::process::id()));
        let dir_a = dir.join("a");
        let dir_b = dir.join("b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        let mut server_chain = chain();
        let block =
            AccountBlock::build(NetworkId::Regtest, server_chain.tip_hash(), 2, 0, vec![]).unwrap();
        server_chain.submit(block).unwrap();
        let shared_chain = Arc::new(Mutex::new(server_chain));
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server_dir = dir.clone();
        let server_chain = Arc::clone(&shared_chain);
        let advertised_peers = Vec::new();
        let server = thread::spawn(move || {
            let mut workers = Vec::new();
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let chain = Arc::clone(&server_chain);
                let data_dir = server_dir.clone();
                let advertised_peers = advertised_peers.clone();
                workers.push(thread::spawn(move || {
                    handle_peer_shared(stream, &chain, &data_dir, &advertised_peers)
                }));
            }
            workers
                .into_iter()
                .all(|worker| worker.join().unwrap().is_ok())
        });

        let peer_a = thread::spawn({
            let dir_a = dir_a.clone();
            move || {
                let mut client = chain();
                sync_from_peer(address, &mut client, &dir_a).unwrap();
                client.height()
            }
        });
        let peer_b = thread::spawn({
            let dir_b = dir_b.clone();
            move || {
                let mut client = chain();
                sync_from_peer(address, &mut client, &dir_b).unwrap();
                client.height()
            }
        });
        assert_eq!(peer_a.join().unwrap(), 1);
        assert_eq!(peer_b.join().unwrap(), 1);
        assert!(server.join().unwrap());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn pushed_block_is_validated_and_persisted_by_peer() {
        let dir = std::env::temp_dir().join(format!("aurora-p2p-push-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let server_chain = chain();
        let server_genesis = server_chain.tip_hash();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server_dir = dir.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let shared_chain = Mutex::new(server_chain);
            let result = handle_peer_shared(stream, &shared_chain, &server_dir, &[]);
            (result, shared_chain.into_inner().unwrap())
        });
        let mut client = chain();
        let block =
            AccountBlock::build(NetworkId::Regtest, client.tip_hash(), 2, 0, vec![]).unwrap();
        client.submit(block.clone()).unwrap();
        propagate_block(address, &client, &block).unwrap();
        let (result, received) = server.join().unwrap();
        assert!(result.is_ok());
        assert_eq!(received.height(), 1);
        assert_ne!(received.tip_hash(), server_genesis);
        let restored =
            aurora_storage::load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Regtest))
                .unwrap();
        assert_eq!(restored.tip_hash(), received.tip_hash());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_reward_block_from_peer_is_rejected_without_persistence() {
        let dir = std::env::temp_dir().join(format!("aurora-p2p-invalid-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let server_chain = chain();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server_dir = dir.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let shared_chain = Mutex::new(server_chain);
            let result = handle_peer_shared(stream, &shared_chain, &server_dir, &[]);
            (result, shared_chain.into_inner().unwrap())
        });
        let client = chain();
        let bad = AccountBlock::build_with_miner(
            NetworkId::Regtest,
            client.tip_hash(),
            2,
            0,
            Some(aurora_core::accounts::AccountId([0x77; 32])),
            vec![],
        )
        .unwrap();
        assert!(propagate_block(address, &client, &bad).is_err());
        let (result, unchanged) = server.join().unwrap();
        assert!(result.is_err());
        assert_eq!(unchanged.height(), 0);
        assert!(fs::read_dir(&dir).unwrap().next().is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn peer_list_encoding_roundtrips_and_rejects_malformed_addresses() {
        let peers = vec![
            "127.0.0.1:18444".parse().unwrap(),
            "[::1]:18445".parse().unwrap(),
        ];
        assert_eq!(
            decode_peer_list(&encode_peer_list(&peers).unwrap()).unwrap(),
            peers
        );

        let mut invalid = vec![PEERS_REPLY];
        invalid.extend_from_slice(&1u16.to_be_bytes());
        invalid.push(4);
        invalid.extend_from_slice(&[0, 0, 0, 0]);
        invalid.extend_from_slice(&18444u16.to_be_bytes());
        assert!(decode_peer_list(&invalid).is_err());

        let excessive = vec!["127.0.0.1:18444".parse().unwrap(); MAX_ADVERTISED_PEERS + 1];
        assert!(encode_peer_list(&excessive).is_err());
        let mut excessive_count = vec![PEERS_REPLY];
        excessive_count.extend_from_slice(&((MAX_ADVERTISED_PEERS as u16) + 1).to_be_bytes());
        assert!(decode_peer_list(&excessive_count).is_err());
    }

    #[test]
    fn peer_can_discover_bounded_static_bootstrap_addresses() {
        let server_chain = chain();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let advertised = vec!["127.0.0.1:18444".parse().unwrap()];
        let expected = advertised.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let shared_chain = Mutex::new(server_chain);
            handle_peer_shared(stream, &shared_chain, Path::new("."), &advertised)
        });
        let discovered = discover_peers(address, &chain()).unwrap();
        assert_eq!(discovered, expected);
        assert!(server.join().unwrap().is_ok());
    }

    #[test]
    fn transient_peer_connection_errors_are_retried_but_protocol_errors_are_not() {
        let mut attempts = 0;
        let result = retry_peer_operation(|| {
            attempts += 1;
            if attempts < 2 {
                Err(P2pError::Io(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "temporary refusal",
                )))
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);

        attempts = 0;
        let result: Result<(), P2pError> = retry_peer_operation(|| {
            attempts += 1;
            Err(P2pError::Protocol("incompatible peer"))
        });
        assert!(result.is_err());
        assert_eq!(attempts, 1);
    }

    #[test]
    fn session_limits_bound_aggregate_bytes_and_request_count() {
        let mut bytes = MAX_SESSION_BYTES - 1;
        assert!(add_session_bytes(&mut bytes, 1).is_ok());
        assert!(add_session_bytes(&mut bytes, 1).is_err());

        let mut requests = MAX_SESSION_REQUESTS - 1;
        assert!(add_session_request(&mut requests).is_ok());
        assert!(add_session_request(&mut requests).is_err());
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
    fn oversized_untrusted_frame_is_rejected_before_session_processing() {
        let dir = std::env::temp_dir().join(format!("aurora-p2p-oversized-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server_dir = dir.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_peer_shared(stream, &Mutex::new(chain()), &server_dir, &[])
        });

        let mut client = TcpStream::connect(address).unwrap();
        client
            .write_all(&((MAX_FRAME as u32) + 1).to_be_bytes())
            .unwrap();
        let result = server.join().unwrap();
        assert!(matches!(
            result,
            Err(P2pError::Protocol("frame length outside limit"))
        ));
        assert!(fs::read_dir(&dir).unwrap().next().is_none());
        fs::remove_dir_all(&dir).unwrap();
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
