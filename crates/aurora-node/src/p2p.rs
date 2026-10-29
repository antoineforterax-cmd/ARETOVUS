//! Small bounded TCP sync protocol for the experimental testnet.
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub use aurora_consensus::testnet::{
    protocol_fingerprint, TESTNET_MAX_FUTURE_BLOCK_SECS as MAX_FUTURE_BLOCK_SECS,
};
use aurora_consensus::{
    account_block::AccountBlock,
    account_fork_chain::{AccountForkChain, AccountForkChainError},
};
#[cfg(test)]
use aurora_core::hash::sha256_plain;
use aurora_core::{
    accounts::{AccountId, AccountTransaction, POST_QUANTUM_ACCOUNT_TX_VERSION},
    codec::{CanonicalDecode, CanonicalEncode},
    hash::Hash256,
    merkle::merkle_proof,
    network::NetworkId,
};
use aurora_storage::save_snapshot;

const MAX_FRAME: usize = 4 * 1024 * 1024;
const PAGE_BLOCKS: usize = 64;
pub const MAX_DISCOVERED_PEERS: usize = 64;
const MAX_ADVERTISED_PEERS: usize = 32;
/// Hard aggregate cap per peer session, independent of per-frame limits.
const MAX_SESSION_BYTES: usize = 256 * 1024 * 1024;
const MAX_SESSION_REQUESTS: usize = 16_384;
const MAX_SESSION_DURATION: Duration = Duration::from_secs(15 * 60);
const IO_TIMEOUT: Duration = Duration::from_secs(10);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_ACTIVE_PER_IP: usize = 4;
const PEER_RETRY_ATTEMPTS: usize = 3;
const PEER_RETRY_BASE_DELAY: Duration = Duration::from_millis(100);
const PEER_POLL_INTERVAL: Duration = Duration::from_secs(10);
const HELLO: u8 = 1;
const GET_BLOCKS: u8 = 2;
const SUBMIT_BLOCK: u8 = 3;
const BLOCK_ACCEPTED: u8 = 0x83;
const BLOCKS_REPLY: u8 = 0x82;
const ERROR_REPLY: u8 = 0xff;
const GET_PEERS: u8 = 4;
const PEERS_REPLY: u8 = 0x84;
const WALLET_HELLO: u8 = 5;
const WALLET_HELLO_REPLY: u8 = 0x85;
const GET_ACCOUNT: u8 = 6;
const ACCOUNT_REPLY: u8 = 0x86;
const SUBMIT_TX: u8 = 7;
const TX_ACCEPTED: u8 = 0x87;
const GET_MEMPOOL: u8 = 8;
const MEMPOOL_REPLY: u8 = 0x88;
const TX_REJECTED: u8 = 0x89;
const GET_TX_STATUS: u8 = 9;
const TX_STATUS_REPLY: u8 = 0x8a;
const GET_HEADERS: u8 = 10;
const HEADERS_REPLY: u8 = 0x8b;
const GET_TX_PROOF: u8 = 11;
const TX_PROOF_REPLY: u8 = 0x8c;
const MAX_HEADERS_PAGE: usize = 64;
const MAX_MEMPOOL_TXS: usize = 4_096;
const MAX_MEMPOOL_BYTES: usize = 32 * 1024 * 1024;
const MAX_MEMPOOL_PAGE_TXS: usize = 700;
const MAX_MEMPOOL_PAGES: usize = 16;

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

fn check_block_timestamp_at(timestamp: u64, now: u64) -> Result<(), P2pError> {
    if timestamp > now.saturating_add(MAX_FUTURE_BLOCK_SECS) {
        return Err(P2pError::Protocol(
            "block timestamp exceeds local future allowance",
        ));
    }
    Ok(())
}

/// Reject a testnet block whose claimed time is too far ahead of this node's clock.
/// This is a time-dependent network admission rule; persisted replay remains deterministic.
pub fn reject_future_block_timestamp(timestamp: u64) -> Result<(), P2pError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| P2pError::Protocol("local clock is before UNIX epoch"))?
        .as_secs();
    check_block_timestamp_at(timestamp, now)
}

type PeerCounts = Arc<Mutex<HashMap<IpAddr, usize>>>;

struct PeerLease {
    ip: IpAddr,
    counts: PeerCounts,
}

impl PeerLease {
    fn acquire(ip: IpAddr, counts: &PeerCounts) -> Result<Option<Self>, P2pError> {
        let mut active = counts.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
        let count = active.entry(ip).or_default();
        if *count >= MAX_ACTIVE_PER_IP {
            return Ok(None);
        }
        *count += 1;
        Ok(Some(Self {
            ip,
            counts: Arc::clone(counts),
        }))
    }
}

impl Drop for PeerLease {
    fn drop(&mut self) {
        if let Ok(mut active) = self.counts.lock() {
            if let Some(count) = active.get_mut(&self.ip) {
                *count -= 1;
                if *count == 0 {
                    active.remove(&self.ip);
                }
            }
        }
    }
}

const PEER_WORKERS: usize = 8;
const MAX_PENDING_HANDSHAKES: usize = 32;
const MAX_READY_HANDSHAKES: usize = 8;
const HANDSHAKE_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Fixed-size preface storage: four length bytes and at most a 73-byte HELLO.
struct PendingHandshake {
    stream: TcpStream,
    lease: PeerLease,
    accepted: Instant,
    bytes: [u8; 77],
    filled: usize,
}

impl PendingHandshake {
    fn new(stream: TcpStream, lease: PeerLease) -> Result<Self, P2pError> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            lease,
            accepted: Instant::now(),
            bytes: [0; 77],
            filled: 0,
        })
    }

    fn poll_at(&mut self, now: Instant) -> Result<Option<Vec<u8>>, P2pError> {
        if now.duration_since(self.accepted) >= HANDSHAKE_TIMEOUT {
            return Err(P2pError::Protocol("handshake deadline exceeded"));
        }
        loop {
            let end = if self.filled < 4 {
                4
            } else {
                let size = u32::from_be_bytes(self.bytes[..4].try_into().unwrap()) as usize;
                if size != 1 && size != 73 {
                    return Err(P2pError::Protocol("invalid handshake length"));
                }
                4 + size
            };
            if self.filled == end {
                let hello = &self.bytes[4..end];
                if !matches!((hello.len(), hello[0]), (1, WALLET_HELLO) | (73, HELLO)) {
                    return Err(P2pError::Protocol("invalid handshake type"));
                }
                return Ok(Some(hello.to_vec()));
            }
            match self.stream.read(&mut self.bytes[self.filled..end]) {
                Ok(0) => return Err(P2pError::Protocol("peer closed during handshake")),
                Ok(count) => self.filled += count,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
}

struct ReadyPeer {
    stream: TcpStream,
    hello: Vec<u8>,
    _lease: PeerLease,
    accepted: Instant,
}

/// No socket reads block this dispatcher, and both application queues are bounded.
fn dispatch_handshakes(
    listener: TcpListener,
    ready: std::sync::mpsc::SyncSender<ReadyPeer>,
    counts: PeerCounts,
) -> Result<(), P2pError> {
    listener.set_nonblocking(true)?;
    let mut pending = Vec::<PendingHandshake>::with_capacity(MAX_PENDING_HANDSHAKES);
    loop {
        let mut index = 0;
        while index < pending.len() {
            match pending[index].poll_at(Instant::now()) {
                Ok(None) => index += 1,
                Err(_) => {
                    pending.swap_remove(index);
                }
                Ok(Some(hello)) => {
                    let peer = pending.swap_remove(index);
                    peer.stream.set_nonblocking(false)?;
                    let task = ReadyPeer {
                        stream: peer.stream,
                        hello,
                        _lease: peer.lease,
                        accepted: peer.accepted,
                    };
                    match ready.try_send(task) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {}
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            return Err(P2pError::Protocol("all peer workers stopped"));
                        }
                    }
                }
            }
        }
        // Limit accepts per pass so a continuous connection flood cannot starve polling.
        for _ in 0..MAX_PENDING_HANDSHAKES {
            match listener.accept() {
                Ok((stream, remote)) => {
                    if pending.len() >= MAX_PENDING_HANDSHAKES {
                        continue;
                    }
                    if let Some(lease) = PeerLease::acquire(remote.ip(), &counts)? {
                        pending.push(PendingHandshake::new(stream, lease)?);
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
        thread::sleep(HANDSHAKE_POLL_INTERVAL);
    }
}

fn set_timeouts(stream: &TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    Ok(())
}
fn read_exact_before_deadline(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    started: Instant,
    budget: Duration,
) -> io::Result<()> {
    let mut cursor = 0;
    while cursor < bytes.len() {
        let remaining = budget.checked_sub(started.elapsed()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::TimedOut, "frame read deadline exceeded")
        })?;
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "frame read deadline exceeded",
            ));
        }
        stream.set_read_timeout(Some(remaining))?;
        match stream.read(&mut bytes[cursor..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "peer closed an incomplete frame",
                ))
            }
            Ok(count) => cursor += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, P2pError> {
    let original_timeout = stream.read_timeout()?;
    let budget = original_timeout.unwrap_or(IO_TIMEOUT).min(IO_TIMEOUT);
    let started = Instant::now();
    let result = (|| {
        let mut raw_len = [0; 4];
        read_exact_before_deadline(stream, &mut raw_len, started, budget)?;
        let len = u32::from_be_bytes(raw_len) as usize;
        if len == 0 || len > MAX_FRAME {
            return Err(P2pError::Protocol("frame length outside limit"));
        }
        let mut frame = vec![0; len];
        read_exact_before_deadline(stream, &mut frame, started, budget)?;
        Ok(frame)
    })();
    stream.set_read_timeout(original_timeout)?;
    result
}
fn write_all_before_deadline(
    stream: &mut TcpStream,
    bytes: &[u8],
    started: Instant,
    budget: Duration,
) -> io::Result<()> {
    let mut cursor = 0;
    while cursor < bytes.len() {
        let remaining = budget.checked_sub(started.elapsed()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::TimedOut, "frame write deadline exceeded")
        })?;
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "frame write deadline exceeded",
            ));
        }
        stream.set_write_timeout(Some(remaining))?;
        match stream.write(&bytes[cursor..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "peer stopped accepting frame bytes",
                ))
            }
            Ok(count) => cursor += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn write_frame(stream: &mut TcpStream, frame: &[u8]) -> Result<(), P2pError> {
    if frame.is_empty() || frame.len() > MAX_FRAME {
        return Err(P2pError::Protocol("outbound frame exceeds limit"));
    }
    let original_timeout = stream.write_timeout()?;
    let budget = original_timeout.unwrap_or(IO_TIMEOUT).min(IO_TIMEOUT);
    let started = Instant::now();
    let result = (|| {
        write_all_before_deadline(stream, &(frame.len() as u32).to_be_bytes(), started, budget)?;
        write_all_before_deadline(stream, frame, started, budget)?;
        stream.flush()?;
        Ok(())
    })();
    stream.set_write_timeout(original_timeout)?;
    result
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

pub fn decode_peer_list(frame: &[u8]) -> Result<Vec<SocketAddr>, P2pError> {
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
fn build_blocks_page(chain: &AccountForkChain, start: u64) -> Result<Vec<u8>, P2pError> {
    let total = u64::try_from(chain.known_block_count())
        .map_err(|_| P2pError::Protocol("block inventory exceeds u64"))?;
    if start > total {
        return Err(P2pError::Protocol("block cursor out of range"));
    }
    let mut frame = vec![BLOCKS_REPLY];
    frame.extend_from_slice(&total.to_be_bytes());
    let mut count = 0u32;
    let count_offset = frame.len();
    frame.extend_from_slice(&count.to_be_bytes());
    for block in chain.arrival_block_page(
        usize::try_from(start).map_err(|_| P2pError::Protocol("block cursor exceeds usize"))?,
        PAGE_BLOCKS,
    ) {
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
    if start < total && count == 0 {
        return Err(P2pError::Protocol("block exceeds transport frame limit"));
    }
    frame[count_offset..count_offset + 4].copy_from_slice(&count.to_be_bytes());
    Ok(frame)
}
pub fn read_blocks(frame: &[u8]) -> Result<(u64, Vec<AccountBlock>), P2pError> {
    if frame.len() < 13 || frame[0] != BLOCKS_REPLY {
        return Err(P2pError::Protocol("invalid block page"));
    }
    let total = u64::from_be_bytes(frame[1..9].try_into().expect("fixed slice"));
    let count = u32::from_be_bytes(frame[9..13].try_into().expect("fixed slice")) as usize;
    if count > PAGE_BLOCKS {
        return Err(P2pError::Protocol("peer inventory exceeds limit"));
    }
    let mut cursor: usize = 13;
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
#[cfg(test)]
fn handle_peer_shared(
    mut stream: TcpStream,
    chain: &Mutex<AccountForkChain>,
    mempool: &Mutex<Vec<AccountTransaction>>,
    data_dir: &Path,
    advertised_peers: &[SocketAddr],
) -> Result<(), P2pError> {
    set_timeouts(&stream)?;
    stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
    let hello = read_frame(&mut stream)?;
    handle_peer_after_hello(stream, chain, mempool, data_dir, advertised_peers, hello)
}

fn handle_peer_after_hello(
    mut stream: TcpStream,
    chain: &Mutex<AccountForkChain>,
    mempool: &Mutex<Vec<AccountTransaction>>,
    data_dir: &Path,
    advertised_peers: &[SocketAddr],
    hello: Vec<u8>,
) -> Result<(), P2pError> {
    set_timeouts(&stream)?;
    let session_started = Instant::now();
    let mut session_bytes = 0usize;
    let mut session_requests = 0usize;
    add_session_bytes(&mut session_bytes, hello.len() + 4)?;
    {
        let chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
        if hello.as_slice() == [WALLET_HELLO] {
            let genesis = chain.blocks().next().expect("genesis exists");
            if genesis.header.network != NetworkId::Testnet {
                let _ = write_frame(&mut stream, &[ERROR_REPLY]);
                return Err(P2pError::Protocol(
                    "wallet RPC is available only on testnet",
                ));
            }
            let mut reply = vec![WALLET_HELLO_REPLY, NetworkId::Testnet.to_byte()];
            reply.extend_from_slice(protocol_fingerprint(&chain).as_bytes());
            reply.extend_from_slice(&chain.height().to_be_bytes());
            reply.extend_from_slice(chain.tip_hash().as_bytes());
            write_frame(&mut stream, &reply)?;
            add_session_bytes(&mut session_bytes, reply.len() + 4)?;
        } else {
            if let Err(error) = check_hello(&hello, &chain) {
                let _ = write_frame(&mut stream, &[ERROR_REPLY]);
                return Err(error);
            }
            if 77 > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                return Err(P2pError::Protocol("peer session byte budget exceeded"));
            }
            send_hello(&mut stream, &chain)?;
            add_session_bytes(&mut session_bytes, 77)?;
        }
    }
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
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
            Some(&GET_BLOCKS) if request.len() == 9 => {
                let start = u64::from_be_bytes(request[1..9].try_into().expect("fixed slice"));
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
            Some(&GET_ACCOUNT) if request.len() == 33 => {
                let id = AccountId(request[1..33].try_into().expect("fixed account id"));
                let account = {
                    let chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                    let pool = mempool.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                    let mut state = chain.state().clone();
                    for tx in pool.iter() {
                        let _ = state.apply_transaction_cached(tx);
                    }
                    state.account(id)
                };
                let mut reply = vec![ACCOUNT_REPLY];
                reply.extend_from_slice(&account.next_nonce.to_be_bytes());
                reply.extend_from_slice(&account.balance.to_units().to_be_bytes());
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, reply.len() + 4)?;
            }
            Some(&SUBMIT_TX) if request.len() > 1 => {
                let tx = AccountTransaction::from_canonical_bytes(&request[1..])
                    .map_err(|_| P2pError::Protocol("invalid canonical transaction"))?;
                let admission = (|| -> Result<(Hash256, usize), u8> {
                    if tx.network != NetworkId::Testnet
                        || tx.version != POST_QUANTUM_ACCOUNT_TX_VERSION
                        || tx.check_stateless().is_err()
                    {
                        return Err(1);
                    }
                    let txid = tx.txid().map_err(|_| 2u8)?;
                    let chain = chain.lock().map_err(|_| 2u8)?;
                    let mut pool = mempool.lock().map_err(|_| 2u8)?;
                    if pool.iter().any(|known| known.txid().ok() == Some(txid)) {
                        return Ok((txid, pool.len()));
                    }
                    if pool.len() >= MAX_MEMPOOL_TXS {
                        return Err(3);
                    }
                    let pool_bytes = pool
                        .iter()
                        .map(|known| {
                            known
                                .to_canonical_bytes()
                                .map(|b| b.len())
                                .unwrap_or(usize::MAX)
                        })
                        .try_fold(0usize, usize::checked_add)
                        .ok_or(3u8)?;
                    let encoded_size = tx.to_canonical_bytes().map_err(|_| 1u8)?.len();
                    if pool_bytes.saturating_add(encoded_size) > MAX_MEMPOOL_BYTES {
                        return Err(3);
                    }
                    let mut candidate = chain.state().clone();
                    for pending in pool.iter() {
                        candidate
                            .apply_transaction_cached(pending)
                            .map_err(|_| 2u8)?;
                    }
                    candidate.apply_transaction_cached(&tx).map_err(|_| 2u8)?;
                    pool.push(tx);
                    Ok((txid, pool.len()))
                })();
                let reply = match admission {
                    Ok((txid, count)) => {
                        let mut reply = vec![TX_ACCEPTED];
                        reply.extend_from_slice(txid.as_bytes());
                        reply.extend_from_slice(&(count as u32).to_be_bytes());
                        reply
                    }
                    Err(code) => vec![TX_REJECTED, code],
                };
                if reply.len() > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, reply.len() + 4)?;
            }
            Some(&GET_MEMPOOL) if request.len() == 5 => {
                let offset =
                    u32::from_be_bytes(request[1..5].try_into().expect("fixed offset")) as usize;
                if offset > MAX_MEMPOOL_TXS {
                    return Err(P2pError::Protocol("mempool offset exceeds limit"));
                }
                let pool = mempool.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                let mut reply = vec![MEMPOOL_REPLY, 0, 0];
                let mut count = 0u16;
                for tx in pool.iter().skip(offset).take(MAX_MEMPOOL_PAGE_TXS) {
                    let encoded = tx
                        .to_canonical_bytes()
                        .map_err(|_| P2pError::Protocol("mempool transaction encoding failed"))?;
                    if reply.len().saturating_add(4 + encoded.len()) > MAX_FRAME {
                        break;
                    }
                    reply.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
                    reply.extend_from_slice(&encoded);
                    count += 1;
                }
                reply[1..3].copy_from_slice(&count.to_be_bytes());
                drop(pool);
                let sent = reply.len() + 4;
                if sent > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, sent)?;
            }
            Some(&GET_TX_STATUS) if request.len() == 33 => {
                let txid = Hash256::from_bytes(request[1..33].try_into().expect("fixed hash"));
                let confirmed = chain
                    .lock()
                    .map_err(|_| P2pError::ChainLockPoisoned)?
                    .canonical_transaction(txid)
                    .is_some();
                let pending = if confirmed {
                    false
                } else {
                    mempool
                        .lock()
                        .map_err(|_| P2pError::ChainLockPoisoned)?
                        .iter()
                        .any(|tx| tx.txid().ok() == Some(txid))
                };
                let reply = vec![
                    TX_STATUS_REPLY,
                    if confirmed {
                        2
                    } else if pending {
                        1
                    } else {
                        0
                    },
                ];
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, reply.len() + 4)?;
            }
            Some(&GET_HEADERS) if request.len() == 11 => {
                let start = u64::from_be_bytes(request[1..9].try_into().expect("fixed height"));
                let limit =
                    u16::from_be_bytes(request[9..11].try_into().expect("fixed limit")) as usize;
                if limit == 0 || limit > MAX_HEADERS_PAGE {
                    return Err(P2pError::Protocol("header page limit outside range"));
                }
                let chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                let tip_height = chain.height();
                if start > tip_height.saturating_add(1) {
                    return Err(P2pError::Protocol("header page starts beyond chain tip"));
                }
                let mut reply = vec![HEADERS_REPLY];
                reply.extend_from_slice(&tip_height.to_be_bytes());
                reply.extend_from_slice(&start.to_be_bytes());
                let count_offset = reply.len();
                reply.extend_from_slice(&0u16.to_be_bytes());
                let mut count = 0u16;
                for block in chain.canonical_block_page(start, limit) {
                    let encoded = block
                        .header
                        .to_canonical_bytes()
                        .map_err(|_| P2pError::Protocol("header encoding failed"))?;
                    reply.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
                    reply.extend_from_slice(&encoded);
                    count += 1;
                }
                reply[count_offset..count_offset + 2].copy_from_slice(&count.to_be_bytes());
                drop(chain);
                let sent = reply.len() + 4;
                if sent > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &reply)?;
                add_session_bytes(&mut session_bytes, sent)?;
            }
            Some(&GET_TX_PROOF) if request.len() == 33 => {
                let txid = Hash256::from_bytes(request[1..33].try_into().expect("fixed txid"));
                let confirmed = chain
                    .lock()
                    .map_err(|_| P2pError::ChainLockPoisoned)?
                    .canonical_transaction(txid)
                    .map(|(height, index, block)| (height, index, block.clone()));
                let found = if let Some((height, index, block)) = confirmed {
                    let txids = block
                        .transactions
                        .iter()
                        .map(|tx| tx.txid())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| P2pError::Protocol("transaction id encoding failed"))?;
                    let proof = merkle_proof(&txids, index)
                        .map_err(|_| P2pError::Protocol("cannot build transaction proof"))?;
                    Some((height, block.header, proof))
                } else {
                    None
                };
                let mut reply = vec![TX_PROOF_REPLY];
                if let Some((height, header, proof)) = found {
                    reply.push(1);
                    reply.extend_from_slice(&height.to_be_bytes());
                    let header = header
                        .to_canonical_bytes()
                        .map_err(|_| P2pError::Protocol("header encoding failed"))?;
                    reply.extend_from_slice(&(header.len() as u16).to_be_bytes());
                    reply.extend_from_slice(&header);
                    reply.extend_from_slice(&proof.index.to_be_bytes());
                    reply.extend_from_slice(&proof.leaf_count.to_be_bytes());
                    reply.push(proof.siblings.len() as u8);
                    for sibling in proof.siblings {
                        reply.extend_from_slice(sibling.as_bytes());
                    }
                } else {
                    reply.push(0);
                }
                let sent = reply.len() + 4;
                if sent > MAX_SESSION_BYTES.saturating_sub(session_bytes) {
                    return Err(P2pError::Protocol("peer session byte budget exceeded"));
                }
                write_frame(&mut stream, &reply)?;
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
                reject_future_block_timestamp(block.header.timestamp)?;
                let mut chain = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
                let mut candidate = chain.clone();
                match candidate.submit(block) {
                    Ok(_) => {
                        save_snapshot(data_dir, &candidate)?;
                        *chain = candidate;
                        revalidate_mempool(&chain, mempool)?;
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

fn revalidate_mempool(
    chain: &AccountForkChain,
    mempool: &Mutex<Vec<AccountTransaction>>,
) -> Result<(), P2pError> {
    let mut pool = mempool.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
    let mut pending = Vec::with_capacity(pool.len());
    let mut state = chain.state().clone();
    for tx in pool.drain(..) {
        if state.apply_transaction_cached(&tx).is_ok() {
            pending.push(tx);
        }
    }
    *pool = pending;
    Ok(())
}

/// Pull pending transactions from a configured peer without trusting its admission decisions.
fn sync_mempool_peer(
    address: SocketAddr,
    chain: &Mutex<AccountForkChain>,
    mempool: &Mutex<Vec<AccountTransaction>>,
) -> Result<usize, P2pError> {
    let handshake_chain = chain
        .lock()
        .map_err(|_| P2pError::ChainLockPoisoned)?
        .clone();
    let incoming = fetch_mempool(address, &handshake_chain)?;
    let current = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
    let mut pool = mempool.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
    let mut state = current.state().clone();
    let mut known = HashSet::with_capacity(pool.len());
    let mut pool_bytes = 0usize;
    for pending in pool.iter() {
        known.insert(
            pending
                .txid()
                .map_err(|_| P2pError::Protocol("local mempool transaction encoding failed"))?,
        );
        pool_bytes = pool_bytes
            .checked_add(
                pending
                    .to_canonical_bytes()
                    .map_err(|_| P2pError::Protocol("local mempool encoding failed"))?
                    .len(),
            )
            .ok_or(P2pError::Protocol("local mempool size overflow"))?;
        state
            .apply_transaction_cached(pending)
            .map_err(|_| P2pError::Protocol("local mempool state invalid"))?;
    }
    let mut added = 0;
    for tx in incoming {
        if tx.network != NetworkId::Testnet || tx.version != POST_QUANTUM_ACCOUNT_TX_VERSION {
            continue;
        }
        let id = tx
            .txid()
            .map_err(|_| P2pError::Protocol("peer mempool transaction encoding failed"))?;
        if known.contains(&id) {
            continue;
        }
        let encoded_size = tx
            .to_canonical_bytes()
            .map_err(|_| P2pError::Protocol("peer mempool encoding failed"))?
            .len();
        if pool.len() >= MAX_MEMPOOL_TXS
            || pool_bytes.saturating_add(encoded_size) > MAX_MEMPOOL_BYTES
        {
            break;
        }
        if state.apply_transaction_cached(&tx).is_err() {
            continue;
        }
        pool.push(tx);
        known.insert(id);
        pool_bytes += encoded_size;
        added += 1;
    }
    Ok(added)
}

/// Download without holding the server lock, then merge all validated branches atomically.
fn sync_peer_into_shared(
    address: SocketAddr,
    chain: &Mutex<AccountForkChain>,
    mempool: &Mutex<Vec<AccountTransaction>>,
    data_dir: &Path,
) -> Result<usize, P2pError> {
    let mut candidate = chain
        .lock()
        .map_err(|_| P2pError::ChainLockPoisoned)?
        .clone();
    let downloaded = sync_from_peer_inner(address, &mut candidate, data_dir, false)?;
    if downloaded == 0 {
        return Ok(0);
    }
    let mut current = chain.lock().map_err(|_| P2pError::ChainLockPoisoned)?;
    let mut merged = current.clone();
    let mut added = 0usize;
    for block in candidate.all_blocks_by_height().into_iter().skip(1) {
        match merged.submit((*block).clone()) {
            Ok(_) => added += 1,
            Err(AccountForkChainError::DuplicateBlock) => {}
            Err(error) => return Err(P2pError::Chain(error)),
        }
    }
    if added != 0 {
        save_snapshot(data_dir, &merged)?;
        *current = merged;
        revalidate_mempool(&current, mempool)?;
    }
    Ok(added)
}

pub fn serve(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
    advertised_peers: &[SocketAddr],
) -> Result<(), P2pError> {
    let listener = TcpListener::bind(address)?;
    println!("testnet peer listening on {}", listener.local_addr()?);
    let chain = Arc::new(Mutex::new(chain.clone()));
    let mempool = Arc::new(Mutex::new(Vec::<AccountTransaction>::new()));
    let data_dir = Arc::new(data_dir.to_path_buf());
    let mut peers = advertised_peers.to_vec();
    peers.sort_unstable();
    peers.dedup();
    peers.truncate(MAX_ADVERTISED_PEERS);
    let advertised_peers = Arc::new(peers);
    let peer_counts: PeerCounts = Arc::new(Mutex::new(HashMap::new()));
    if !advertised_peers.is_empty() {
        let sync_chain = Arc::clone(&chain);
        let sync_mempool = Arc::clone(&mempool);
        let sync_data_dir = Arc::clone(&data_dir);
        let sync_peers = Arc::clone(&advertised_peers);
        thread::Builder::new()
            .name("aurora-peer-sync".to_string())
            .spawn(move || loop {
                for peer in sync_peers.iter().copied() {
                    if let Err(error) =
                        sync_peer_into_shared(peer, &sync_chain, &sync_mempool, &sync_data_dir)
                    {
                        eprintln!("background peer sync from {peer} failed: {error}");
                    }
                    if let Err(error) = sync_mempool_peer(peer, &sync_chain, &sync_mempool) {
                        eprintln!("background mempool sync from {peer} failed: {error}");
                    }
                }
                thread::sleep(PEER_POLL_INTERVAL);
            })?;
    }
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<ReadyPeer>(MAX_READY_HANDSHAKES);
    let ready_rx = Arc::new(Mutex::new(ready_rx));
    for worker_id in 0..PEER_WORKERS {
        let chain = Arc::clone(&chain);
        let mempool = Arc::clone(&mempool);
        let data_dir = Arc::clone(&data_dir);
        let advertised_peers = Arc::clone(&advertised_peers);
        let ready_rx = Arc::clone(&ready_rx);
        thread::Builder::new()
            .name(format!("aurora-peer-{worker_id}"))
            .spawn(move || loop {
                let task = {
                    let receiver = match ready_rx.lock() {
                        Ok(receiver) => receiver,
                        Err(_) => return,
                    };
                    match receiver.recv() {
                        Ok(task) => task,
                        Err(_) => return,
                    }
                };
                // Queue residence does not extend the original handshake deadline.
                if task.accepted.elapsed() >= HANDSHAKE_TIMEOUT {
                    continue;
                }
                if let Err(error) = handle_peer_after_hello(
                    task.stream,
                    &chain,
                    &mempool,
                    &data_dir,
                    &advertised_peers,
                    task.hello,
                ) {
                    eprintln!("peer rejected: {error}");
                }
            })?;
    }
    dispatch_handshakes(listener, ready_tx, peer_counts)
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
    sync_from_peer_inner(address, chain, data_dir, true)
}

fn sync_from_peer_inner(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
    persist: bool,
) -> Result<usize, P2pError> {
    sync_from_peer_with_request_limit(
        address,
        chain,
        data_dir,
        persist,
        MAX_SESSION_REQUESTS.saturating_sub(2),
    )
}

fn sync_from_peer_with_request_limit(
    address: SocketAddr,
    chain: &mut AccountForkChain,
    data_dir: &Path,
    persist: bool,
    restart_after_requests: usize,
) -> Result<usize, P2pError> {
    let (mut stream, mut session_bytes) = open_sync_session(address, chain)?;
    let mut session_started = Instant::now();
    let mut session_requests = 0usize;
    let mut cursor = 1u64;
    let mut accepted = 0;
    let mut last_block_hash = None;
    let mut announced_total = 0u64;
    // An arrival inventory contains each block once, even when it is already
    // known locally. Keep this across session rollover: otherwise a peer can
    // repeat one valid block forever while inventing new cursor positions.
    let mut inventory_hashes = HashSet::new();
    inventory_hashes.insert(
        chain
            .blocks()
            .next()
            .expect("genesis exists")
            .header
            .hash()
            .map_err(|_| P2pError::Protocol("block header hash failed"))?,
    );
    loop {
        // A session cannot assume that an inventory fits within its byte, request, or
        // time budget. Reopen before any of those limits and prove that the last
        // applied cursor still names the same block on the new connection.
        if session_bytes > MAX_SESSION_BYTES.saturating_sub(MAX_FRAME + 17)
            || session_requests >= restart_after_requests
            || session_started.elapsed() >= MAX_SESSION_DURATION.saturating_sub(IO_TIMEOUT * 2)
        {
            drop(stream);
            (stream, session_bytes) = open_sync_session(address, chain)?;
            session_started = Instant::now();
            session_requests = 0;
            if let Some(expected_hash) = last_block_hash {
                add_session_request(&mut session_requests)?;
                add_session_bytes(&mut session_bytes, 13)?;
                let mut probe = vec![GET_BLOCKS];
                probe.extend_from_slice(&(cursor - 1).to_be_bytes());
                write_frame(&mut stream, &probe)?;
                let reply = read_frame(&mut stream)?;
                add_session_bytes(&mut session_bytes, reply.len() + 4)?;
                let (total, blocks) = read_blocks(&reply)?;
                if total < cursor
                    || total < announced_total
                    || blocks.first().and_then(|block| block.header.hash().ok())
                        != Some(expected_hash)
                {
                    return Err(P2pError::Protocol("peer inventory changed during sync"));
                }
            }
        }
        add_session_request(&mut session_requests)?;
        add_session_bytes(&mut session_bytes, 13)?;
        let mut request = vec![GET_BLOCKS];
        request.extend_from_slice(&cursor.to_be_bytes());
        write_frame(&mut stream, &request)?;
        let page = read_frame(&mut stream)?;
        add_session_bytes(&mut session_bytes, page.len() + 4)?;
        let (total, blocks) = read_blocks(&page)?;
        if total < announced_total {
            return Err(P2pError::Protocol("peer inventory shrank during sync"));
        }
        announced_total = total;
        if total < cursor
            || u64::try_from(blocks.len()).map_err(|_| P2pError::Protocol("page too large"))?
                > total.saturating_sub(cursor)
        {
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
        // Check the complete page before cloning or publishing candidate state.
        // The continuity probe above intentionally rereads one position and is
        // not a new inventory entry, so it does not participate in this set.
        let mut page_hashes = HashSet::with_capacity(blocks.len());
        for block in &blocks {
            let hash = block
                .header
                .hash()
                .map_err(|_| P2pError::Protocol("block header hash failed"))?;
            if inventory_hashes.contains(&hash) || !page_hashes.insert(hash) {
                return Err(P2pError::Protocol("peer inventory repeats a block"));
            }
        }
        last_block_hash = Some(
            blocks
                .last()
                .expect("nonempty page")
                .header
                .hash()
                .map_err(|_| P2pError::Protocol("block header hash failed"))?,
        );
        cursor = cursor
            .checked_add(
                u64::try_from(blocks.len()).map_err(|_| P2pError::Protocol("page too large"))?,
            )
            .ok_or(P2pError::Protocol("cursor overflow"))?;
        let mut candidate = chain.clone();
        let mut page_accepted = 0usize;
        for block in blocks {
            reject_future_block_timestamp(block.header.timestamp)?;
            match candidate.submit(block) {
                Ok(_) => page_accepted += 1,
                Err(AccountForkChainError::DuplicateBlock) => {}
                Err(error) => return Err(P2pError::Chain(error)),
            }
        }
        if page_accepted != 0 {
            if persist {
                save_snapshot(data_dir, &candidate)?;
            }
            *chain = candidate;
            accepted += page_accepted;
        }
        // Never reserve from the peer's announced total. Retained entries name
        // validated blocks in the local chain; only this page (<= PAGE_BLOCKS)
        // is held provisionally. Invalid pages cannot grow the retained set.
        inventory_hashes.extend(page_hashes);
        debug_assert!(inventory_hashes.len() <= chain.known_block_count());
        if cursor >= total {
            break;
        }
    }
    Ok(accepted)
}

fn open_sync_session(
    address: SocketAddr,
    chain: &AccountForkChain,
) -> Result<(TcpStream, usize), P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    let mut session_bytes = 77usize;
    send_hello(&mut stream, chain)?;
    let reply = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, reply.len() + 4)?;
    if reply.first() == Some(&ERROR_REPLY) {
        return Err(P2pError::Protocol("peer rejected network identity"));
    }
    check_hello(&reply, chain)?;
    Ok((stream, session_bytes))
}

/// Fetch bounded pages of pending transactions from a compatible testnet peer.
pub fn fetch_mempool(
    address: SocketAddr,
    chain: &AccountForkChain,
) -> Result<Vec<AccountTransaction>, P2pError> {
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    set_timeouts(&stream)?;
    let mut session_bytes = 77usize;
    let mut session_requests = 0usize;
    send_hello(&mut stream, chain)?;
    let hello = read_frame(&mut stream)?;
    add_session_bytes(&mut session_bytes, hello.len() + 4)?;
    if hello.first() == Some(&ERROR_REPLY) {
        return Err(P2pError::Protocol("peer rejected network identity"));
    }
    check_hello(&hello, chain)?;
    let mut transactions = Vec::new();
    let mut pages = 0usize;
    loop {
        pages += 1;
        if pages > MAX_MEMPOOL_PAGES {
            return Err(P2pError::Protocol("peer mempool has too many pages"));
        }
        if transactions.len() > MAX_MEMPOOL_TXS {
            return Err(P2pError::Protocol("peer mempool exceeds total limit"));
        }
        add_session_request(&mut session_requests)?;
        add_session_bytes(&mut session_bytes, 9)?;
        let mut request = vec![GET_MEMPOOL];
        request.extend_from_slice(&(transactions.len() as u32).to_be_bytes());
        write_frame(&mut stream, &request)?;
        let frame = read_frame(&mut stream)?;
        add_session_bytes(&mut session_bytes, frame.len() + 4)?;
        if frame.len() < 3 || frame[0] != MEMPOOL_REPLY {
            return Err(P2pError::Protocol("invalid mempool page"));
        }
        let count = u16::from_be_bytes(frame[1..3].try_into().expect("fixed count")) as usize;
        if count > MAX_MEMPOOL_PAGE_TXS
            || transactions.len().saturating_add(count) > MAX_MEMPOOL_TXS
        {
            return Err(P2pError::Protocol("mempool page exceeds transaction limit"));
        }
        let mut cursor = 3usize;
        for _ in 0..count {
            let length_end = cursor
                .checked_add(4)
                .ok_or(P2pError::Protocol("mempool length overflow"))?;
            let length_bytes = frame
                .get(cursor..length_end)
                .ok_or(P2pError::Protocol("truncated mempool transaction length"))?;
            let length =
                u32::from_be_bytes(length_bytes.try_into().expect("fixed length")) as usize;
            cursor = length_end;
            let end = cursor
                .checked_add(length)
                .ok_or(P2pError::Protocol("mempool length overflow"))?;
            let encoded = frame
                .get(cursor..end)
                .ok_or(P2pError::Protocol("truncated mempool transaction"))?;
            transactions.push(
                AccountTransaction::from_canonical_bytes(encoded)
                    .map_err(|_| P2pError::Protocol("invalid canonical mempool transaction"))?,
            );
            cursor = end;
        }
        if cursor != frame.len() {
            return Err(P2pError::Protocol("trailing mempool page data"));
        }
        if count == 0 {
            break;
        }
    }
    Ok(transactions)
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

    #[test]
    fn future_block_admission_has_exact_boundary_and_handles_overflow() {
        let now = 1_700_000_000;
        assert!(check_block_timestamp_at(now + MAX_FUTURE_BLOCK_SECS, now).is_ok());
        assert!(matches!(
            check_block_timestamp_at(now + MAX_FUTURE_BLOCK_SECS + 1, now),
            Err(P2pError::Protocol(
                "block timestamp exceeds local future allowance"
            ))
        ));
        assert!(check_block_timestamp_at(u64::MAX, u64::MAX).is_ok());
    }

    #[test]
    fn block_inventory_cursor_survives_late_side_branch() {
        let mut chain = chain();
        let genesis = chain.tip_hash();
        let first = AccountBlock::build(NetworkId::Regtest, genesis, 2, 0, vec![]).unwrap();
        let first_hash = first.header.hash().unwrap();
        chain.submit(first).unwrap();
        let second = AccountBlock::build(NetworkId::Regtest, first_hash, 3, 0, vec![]).unwrap();
        let second_hash = second.header.hash().unwrap();
        chain.submit(second).unwrap();
        let before = build_blocks_page(&chain, 1).unwrap();
        let (_, before_blocks) = read_blocks(&before).unwrap();
        let before_hashes: Vec<_> = before_blocks
            .iter()
            .map(|block| block.header.hash().unwrap())
            .collect();
        assert_eq!(before_hashes, vec![first_hash, second_hash]);

        let late = AccountBlock::build(NetworkId::Regtest, genesis, 4, 0, vec![]).unwrap();
        let late_hash = late.header.hash().unwrap();
        chain.submit(late).unwrap();
        let after = build_blocks_page(&chain, 1).unwrap();
        let (_, after_blocks) = read_blocks(&after).unwrap();
        let after_hashes: Vec<_> = after_blocks
            .iter()
            .map(|block| block.header.hash().unwrap())
            .collect();
        assert_eq!(after_hashes, vec![first_hash, second_hash, late_hash]);
    }

    #[test]
    fn sync_reopens_session_and_checks_last_cursor_before_continuing() {
        let mut remote = chain();
        let first =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
        let first_hash = first.header.hash().unwrap();
        remote.submit(first.clone()).unwrap();
        let second = AccountBlock::build(NetworkId::Regtest, first_hash, 3, 0, vec![]).unwrap();
        let second_hash = second.header.hash().unwrap();
        remote.submit(second).unwrap();

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut first_session, _) = listener.accept().unwrap();
            set_timeouts(&first_session).unwrap();
            read_frame(&mut first_session).unwrap();
            send_hello(&mut first_session, &remote).unwrap();
            assert_eq!(read_frame(&mut first_session).unwrap()[0], GET_BLOCKS);
            let bytes = first.to_canonical_bytes().unwrap();
            let mut first_page = vec![BLOCKS_REPLY];
            first_page.extend_from_slice(&3u64.to_be_bytes());
            first_page.extend_from_slice(&1u32.to_be_bytes());
            first_page.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            first_page.extend_from_slice(&bytes);
            write_frame(&mut first_session, &first_page).unwrap();
            drop(first_session);

            let (mut second_session, _) = listener.accept().unwrap();
            set_timeouts(&second_session).unwrap();
            read_frame(&mut second_session).unwrap();
            send_hello(&mut second_session, &remote).unwrap();
            let probe = read_frame(&mut second_session).unwrap();
            assert_eq!(probe[0], GET_BLOCKS);
            assert_eq!(u64::from_be_bytes(probe[1..9].try_into().unwrap()), 1);
            write_frame(&mut second_session, &build_blocks_page(&remote, 1).unwrap()).unwrap();
            let request = read_frame(&mut second_session).unwrap();
            assert_eq!(u64::from_be_bytes(request[1..9].try_into().unwrap()), 2);
            write_frame(&mut second_session, &build_blocks_page(&remote, 2).unwrap()).unwrap();
        });

        let mut client = chain();
        let count =
            sync_from_peer_with_request_limit(address, &mut client, Path::new("."), false, 1)
                .unwrap();
        assert_eq!(count, 2);
        assert_eq!(client.height(), 2);
        assert_eq!(client.tip_hash(), second_hash);
        server.join().unwrap();
    }

    #[test]
    fn shrinking_peer_inventory_is_rejected() {
        let mut remote = chain();
        let first =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
        remote.submit(first.clone()).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &remote).unwrap();
            read_frame(&mut stream).unwrap();
            let encoded = first.to_canonical_bytes().unwrap();
            let mut page = vec![BLOCKS_REPLY];
            page.extend_from_slice(&3u64.to_be_bytes());
            page.extend_from_slice(&1u32.to_be_bytes());
            page.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
            page.extend_from_slice(&encoded);
            write_frame(&mut stream, &page).unwrap();
            read_frame(&mut stream).unwrap();
            let mut shorter = vec![BLOCKS_REPLY];
            shorter.extend_from_slice(&2u64.to_be_bytes());
            shorter.extend_from_slice(&0u32.to_be_bytes());
            write_frame(&mut stream, &shorter).unwrap();
        });
        let mut client = chain();
        assert!(matches!(
            sync_from_peer_with_request_limit(address, &mut client, Path::new("."), false, 10),
            Err(P2pError::Protocol("peer inventory shrank during sync"))
        ));
        assert_eq!(client.height(), 1);
        server.join().unwrap();
    }

    #[test]
    fn changed_peer_cursor_is_rejected_after_reconnect() {
        let mut remote = chain();
        let first =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
        let alternate =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 3, 0, vec![]).unwrap();
        remote.submit(first.clone()).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut first_session, _) = listener.accept().unwrap();
            set_timeouts(&first_session).unwrap();
            read_frame(&mut first_session).unwrap();
            send_hello(&mut first_session, &remote).unwrap();
            read_frame(&mut first_session).unwrap();
            let encoded = first.to_canonical_bytes().unwrap();
            let mut page = vec![BLOCKS_REPLY];
            page.extend_from_slice(&3u64.to_be_bytes());
            page.extend_from_slice(&1u32.to_be_bytes());
            page.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
            page.extend_from_slice(&encoded);
            write_frame(&mut first_session, &page).unwrap();
            drop(first_session);

            let (mut next_session, _) = listener.accept().unwrap();
            set_timeouts(&next_session).unwrap();
            read_frame(&mut next_session).unwrap();
            send_hello(&mut next_session, &remote).unwrap();
            read_frame(&mut next_session).unwrap();
            let encoded = alternate.to_canonical_bytes().unwrap();
            let mut changed = vec![BLOCKS_REPLY];
            changed.extend_from_slice(&3u64.to_be_bytes());
            changed.extend_from_slice(&1u32.to_be_bytes());
            changed.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
            changed.extend_from_slice(&encoded);
            write_frame(&mut next_session, &changed).unwrap();
        });
        let mut client = chain();
        assert!(matches!(
            sync_from_peer_with_request_limit(address, &mut client, Path::new("."), false, 1),
            Err(P2pError::Protocol("peer inventory changed during sync"))
        ));
        assert_eq!(client.height(), 1);
        server.join().unwrap();
    }

    #[test]
    fn repeated_block_in_one_sync_page_is_rejected_before_persistence() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-repeated-page-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut remote = chain();
        let first =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
        remote.submit(first.clone()).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &remote).unwrap();
            read_frame(&mut stream).unwrap();
            let mut page = vec![BLOCKS_REPLY];
            page.extend_from_slice(&3u64.to_be_bytes());
            page.extend_from_slice(&2u32.to_be_bytes());
            let encoded = first.to_canonical_bytes().unwrap();
            for _ in 0..2 {
                page.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
                page.extend_from_slice(&encoded);
            }
            write_frame(&mut stream, &page).unwrap();
        });
        let mut client = chain();
        let original_tip = client.tip_hash();
        assert!(matches!(
            sync_from_peer(address, &mut client, &dir),
            Err(P2pError::Protocol("peer inventory repeats a block"))
        ));
        assert_eq!(client.height(), 0);
        assert_eq!(client.tip_hash(), original_tip);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn repeated_inventory_block_is_rejected_across_pages_and_session_rollover() {
        for restart_after_requests in [10, 1] {
            let mut remote = chain();
            let first =
                AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
            let first_hash = first.header.hash().unwrap();
            remote.submit(first.clone()).unwrap();
            let mut client = chain();
            client.submit(first.clone()).unwrap();
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                set_timeouts(&stream).unwrap();
                read_frame(&mut stream).unwrap();
                send_hello(&mut stream, &remote).unwrap();
                let request = read_frame(&mut stream).unwrap();
                assert_eq!(u64::from_be_bytes(request[1..9].try_into().unwrap()), 1);
                let encoded = first.to_canonical_bytes().unwrap();
                let mut page = vec![BLOCKS_REPLY];
                // An enormous advertised total must not allocate an enormous
                // table or let duplicate entries keep the client busy forever.
                page.extend_from_slice(&u64::MAX.to_be_bytes());
                page.extend_from_slice(&1u32.to_be_bytes());
                page.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
                page.extend_from_slice(&encoded);
                write_frame(&mut stream, &page).unwrap();
                if restart_after_requests == 1 {
                    drop(stream);
                    (stream, _) = listener.accept().unwrap();
                    set_timeouts(&stream).unwrap();
                    read_frame(&mut stream).unwrap();
                    send_hello(&mut stream, &remote).unwrap();
                    let probe = read_frame(&mut stream).unwrap();
                    assert_eq!(u64::from_be_bytes(probe[1..9].try_into().unwrap()), 1);
                    write_frame(&mut stream, &page).unwrap();
                }
                let request = read_frame(&mut stream).unwrap();
                assert_eq!(u64::from_be_bytes(request[1..9].try_into().unwrap()), 2);
                write_frame(&mut stream, &page).unwrap();
            });
            // A block already held locally is legitimate once in the remote
            // inventory. It must not become a bypass for repeated positions.
            assert!(matches!(
                sync_from_peer_with_request_limit(
                    address,
                    &mut client,
                    Path::new("."),
                    false,
                    restart_after_requests
                ),
                Err(P2pError::Protocol("peer inventory repeats a block"))
            ));
            assert_eq!(client.height(), 1);
            assert_eq!(client.tip_hash(), first_hash);
            assert_eq!(client.known_block_count(), 2);
            server.join().unwrap();
        }
    }

    #[test]
    fn sync_accepts_remote_inventory_that_overlaps_local_blocks() {
        let mut remote = chain();
        let first =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 2, 0, vec![]).unwrap();
        remote.submit(first.clone()).unwrap();
        let mut client = chain();
        client.submit(first).unwrap();
        let second =
            AccountBlock::build(NetworkId::Regtest, remote.tip_hash(), 3, 0, vec![]).unwrap();
        remote.submit(second).unwrap();
        let expected_tip = remote.tip_hash();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_peer_shared(
                stream,
                &Mutex::new(remote),
                &Mutex::new(Vec::new()),
                Path::new("."),
                &[],
            )
            .unwrap();
        });
        assert_eq!(
            sync_from_peer_with_request_limit(address, &mut client, Path::new("."), false, 10)
                .unwrap(),
            1
        );
        assert_eq!(client.height(), 2);
        assert_eq!(client.tip_hash(), expected_tip);
        assert_eq!(client.known_block_count(), 3);
        server.join().unwrap();
    }

    #[test]
    fn per_ip_admission_reserves_workers_for_other_peers() {
        let counts: PeerCounts = Arc::new(Mutex::new(HashMap::new()));
        let ip: IpAddr = "192.0.2.1".parse().unwrap();
        let other: IpAddr = "192.0.2.2".parse().unwrap();
        let leases: Vec<_> = (0..MAX_ACTIVE_PER_IP)
            .map(|_| PeerLease::acquire(ip, &counts).unwrap().unwrap())
            .collect();
        assert!(PeerLease::acquire(ip, &counts).unwrap().is_none());
        assert!(PeerLease::acquire(other, &counts).unwrap().is_some());
        drop(leases);
        assert!(PeerLease::acquire(ip, &counts).unwrap().is_some());
    }

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
    fn header_rpc_preserves_page_encoding_and_rejects_out_of_range_height() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_peer_shared(
                stream,
                &Mutex::new(chain()),
                &Mutex::new(Vec::new()),
                &std::env::temp_dir(),
                &[],
            )
        });
        let local = chain();
        let mut client = TcpStream::connect(address).unwrap();
        set_timeouts(&client).unwrap();
        send_hello(&mut client, &local).unwrap();
        check_hello(&read_frame(&mut client).unwrap(), &local).unwrap();
        for start in [0u64, 1] {
            let mut request = vec![GET_HEADERS];
            request.extend_from_slice(&start.to_be_bytes());
            request.extend_from_slice(&1u16.to_be_bytes());
            write_frame(&mut client, &request).unwrap();
            let reply = read_frame(&mut client).unwrap();
            let mut expected = vec![HEADERS_REPLY];
            expected.extend_from_slice(&0u64.to_be_bytes());
            expected.extend_from_slice(&start.to_be_bytes());
            expected.extend_from_slice(&(if start == 0 { 1u16 } else { 0 }).to_be_bytes());
            if start == 0 {
                let encoded = local
                    .blocks()
                    .next()
                    .unwrap()
                    .header
                    .to_canonical_bytes()
                    .unwrap();
                expected.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
                expected.extend_from_slice(&encoded);
            }
            assert_eq!(reply, expected);
        }
        let mut invalid = vec![GET_HEADERS];
        invalid.extend_from_slice(&u64::MAX.to_be_bytes());
        invalid.extend_from_slice(&1u16.to_be_bytes());
        write_frame(&mut client, &invalid).unwrap();
        assert!(matches!(
            server.join().unwrap(),
            Err(P2pError::Protocol("header page starts beyond chain tip"))
        ));
    }

    #[test]
    fn canonical_header_pages_match_selected_branch_and_handle_large_heights() {
        let mut chain = chain();
        let genesis = chain.tip_hash();
        for height in 1..=12 {
            let block =
                AccountBlock::build(NetworkId::Regtest, chain.tip_hash(), height + 1, 0, vec![])
                    .unwrap();
            chain.submit(block).unwrap();
        }
        let check_pages = |chain: &AccountForkChain| {
            for start in [0, 1, 10, 12, 13, 14, u32::MAX as u64 + 1, u64::MAX] {
                for limit in [0, 1, 3, MAX_HEADERS_PAGE] {
                    let actual: Vec<_> = chain
                        .canonical_block_page(start, limit)
                        .map(|block| block.header.hash().unwrap())
                        .collect();
                    let expected: Vec<_> = chain
                        .blocks()
                        .enumerate()
                        .filter(|(height, _)| *height as u64 >= start)
                        .take(limit)
                        .map(|(_, block)| block.header.hash().unwrap())
                        .collect();
                    assert_eq!(actual, expected, "start={start}, limit={limit}");
                }
            }
        };
        check_pages(&chain);
        let mut parent = genesis;
        for height in 1..=13 {
            let block =
                AccountBlock::build(NetworkId::Regtest, parent, height + 100, 0, vec![]).unwrap();
            parent = block.header.hash().unwrap();
            chain.submit(block).unwrap();
        }
        assert_eq!(chain.tip_hash(), parent);
        check_pages(&chain);
    }

    #[test]
    fn background_sync_merges_remote_blocks_without_dropping_local_branch() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-background-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut remote = chain();
        let genesis = remote.tip_hash();
        let remote_block = AccountBlock::build(NetworkId::Regtest, genesis, 2, 0, vec![]).unwrap();
        remote.submit(remote_block).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server_dir = dir.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_peer_shared(
                stream,
                &Mutex::new(remote),
                &Mutex::new(Vec::new()),
                &server_dir,
                &[],
            )
            .unwrap();
        });

        let mut local = chain();
        let local_block = AccountBlock::build(NetworkId::Regtest, genesis, 3, 0, vec![]).unwrap();
        local.submit(local_block).unwrap();
        let shared = Mutex::new(local);
        let mempool = Mutex::new(Vec::new());
        assert_eq!(
            sync_peer_into_shared(address, &shared, &mempool, &dir).unwrap(),
            1
        );
        let current = shared.lock().unwrap();
        assert_eq!(current.known_block_count(), 3);
        let restored =
            aurora_storage::load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Regtest))
                .unwrap();
        assert_eq!(restored.known_block_count(), 3);
        assert_eq!(restored.tip_hash(), current.tip_hash());
        drop(current);
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
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
            let result = handle_peer_shared(
                stream,
                &shared_chain,
                &Mutex::new(Vec::new()),
                &server_dir,
                &[],
            );
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
                    handle_peer_shared(
                        stream,
                        &chain,
                        &Mutex::new(Vec::new()),
                        &data_dir,
                        &advertised_peers,
                    )
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
            let result = handle_peer_shared(
                stream,
                &shared_chain,
                &Mutex::new(Vec::new()),
                &server_dir,
                &[],
            );
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
            let result = handle_peer_shared(
                stream,
                &shared_chain,
                &Mutex::new(Vec::new()),
                &server_dir,
                &[],
            );
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
            handle_peer_shared(
                stream,
                &shared_chain,
                &Mutex::new(Vec::new()),
                Path::new("."),
                &advertised,
            )
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
        oversized.extend_from_slice(&(100_001u64).to_be_bytes());
        oversized.extend_from_slice(&0u32.to_be_bytes());
        assert_eq!(read_blocks(&oversized).unwrap().0, 100_001);

        let mut trailing = vec![BLOCKS_REPLY];
        trailing.extend_from_slice(&1u64.to_be_bytes());
        trailing.extend_from_slice(&0u32.to_be_bytes());
        trailing.push(0);
        assert!(read_blocks(&trailing).is_err());
    }

    #[test]
    fn partial_frame_from_slow_peer_obeys_read_timeout() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            let started = Instant::now();
            let result = read_frame(&mut stream);
            (result, started.elapsed())
        });

        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(&[0, 0]).unwrap();
        let (result, elapsed) = server.join().unwrap();
        assert!(matches!(
            result,
            Err(P2pError::Io(ref error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                )
        ));
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn trickle_fed_frame_cannot_hold_a_peer_session_indefinitely() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            let started = Instant::now();
            (read_frame(&mut stream), started.elapsed())
        });

        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(&8u32.to_be_bytes()).unwrap();
        for _ in 0..8 {
            thread::sleep(Duration::from_millis(50));
            let _ = client.write_all(&[0]);
        }
        let (result, elapsed) = server.join().unwrap();
        assert!(matches!(
            result,
            Err(P2pError::Io(ref error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                )
        ));
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn peer_that_stops_reading_cannot_hold_a_frame_write_indefinitely() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let mut writer = TcpStream::connect(address).unwrap();
        let (_reader, _) = listener.accept().unwrap();
        writer
            .set_write_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let frame = vec![0u8; MAX_FRAME];
        let started = Instant::now();
        let mut timed_out = false;
        for _ in 0..32 {
            match write_frame(&mut writer, &frame) {
                Ok(()) => {}
                Err(P2pError::Io(error))
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    timed_out = true;
                    break;
                }
                Err(error) => panic!("unexpected frame write error: {error}"),
            }
        }
        assert!(timed_out, "peer accepted 128 MiB without reading");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn future_block_in_sync_page_is_rejected_before_persistence() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-future-page-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let server_chain = chain();
        let future = AccountBlock::build(
            NetworkId::Regtest,
            server_chain.tip_hash(),
            u64::MAX,
            0,
            vec![],
        )
        .unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &server_chain).unwrap();
            read_frame(&mut stream).unwrap();
            let bytes = future.to_canonical_bytes().unwrap();
            let mut page = vec![BLOCKS_REPLY];
            page.extend_from_slice(&2u64.to_be_bytes());
            page.extend_from_slice(&1u32.to_be_bytes());
            page.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            page.extend_from_slice(&bytes);
            write_frame(&mut stream, &page).unwrap();
        });

        let mut client = chain();
        let original_tip = client.tip_hash();
        assert!(matches!(
            sync_from_peer(address, &mut client, &dir),
            Err(P2pError::Protocol(
                "block timestamp exceeds local future allowance"
            ))
        ));
        assert_eq!(client.tip_hash(), original_tip);
        assert_eq!(client.height(), 0);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_second_block_does_not_partially_apply_a_sync_page() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-atomic-page-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let server_chain = chain();
        let valid =
            AccountBlock::build(NetworkId::Regtest, server_chain.tip_hash(), 2, 0, vec![]).unwrap();
        let orphan = AccountBlock::build(
            NetworkId::Regtest,
            sha256_plain(b"missing-parent"),
            3,
            0,
            vec![],
        )
        .unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &server_chain).unwrap();
            read_frame(&mut stream).unwrap();
            let mut page = vec![BLOCKS_REPLY];
            page.extend_from_slice(&3u64.to_be_bytes());
            page.extend_from_slice(&2u32.to_be_bytes());
            for block in [valid, orphan] {
                let bytes = block.to_canonical_bytes().unwrap();
                page.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                page.extend_from_slice(&bytes);
            }
            write_frame(&mut stream, &page).unwrap();
        });

        let mut client = chain();
        let original_tip = client.tip_hash();
        assert!(matches!(
            sync_from_peer(address, &mut client, &dir),
            Err(P2pError::Chain(_))
        ));
        assert_eq!(client.height(), 0);
        assert_eq!(client.tip_hash(), original_tip);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn disconnected_peer_does_not_publish_a_partial_sync_page() {
        let dir =
            std::env::temp_dir().join(format!("aurora-p2p-truncated-page-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut server_chain = chain();
        let block =
            AccountBlock::build(NetworkId::Regtest, server_chain.tip_hash(), 2, 0, vec![]).unwrap();
        server_chain.submit(block).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &server_chain).unwrap();
            read_frame(&mut stream).unwrap();
            let page = build_blocks_page(&server_chain, 1).unwrap();
            stream
                .write_all(&(page.len() as u32).to_be_bytes())
                .unwrap();
            stream.write_all(&page[..page.len() - 1]).unwrap();
        });

        let mut client = chain();
        let original_tip = client.tip_hash();
        assert!(matches!(
            sync_from_peer(address, &mut client, &dir),
            Err(P2pError::Io(ref error)) if error.kind() == io::ErrorKind::UnexpectedEof
        ));
        assert_eq!(client.height(), 0);
        assert_eq!(client.tip_hash(), original_tip);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_snapshot_does_not_publish_a_synced_page() {
        let path = std::env::temp_dir().join(format!(
            "aurora-p2p-snapshot-failure-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        fs::write(&path, b"not a directory").unwrap();
        let mut server_chain = chain();
        let block =
            AccountBlock::build(NetworkId::Regtest, server_chain.tip_hash(), 2, 0, vec![]).unwrap();
        server_chain.submit(block).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            set_timeouts(&stream).unwrap();
            read_frame(&mut stream).unwrap();
            send_hello(&mut stream, &server_chain).unwrap();
            read_frame(&mut stream).unwrap();
            write_frame(&mut stream, &build_blocks_page(&server_chain, 1).unwrap()).unwrap();
        });

        let mut client = chain();
        let original_tip = client.tip_hash();
        assert!(matches!(
            sync_from_peer(address, &mut client, &path),
            Err(P2pError::Storage(_))
        ));
        assert_eq!(client.height(), 0);
        assert_eq!(client.tip_hash(), original_tip);
        server.join().unwrap();
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn deterministic_decoder_fuzz_smoke_rejects_arbitrary_bytes_without_panicking() {
        use aurora_consensus::experimental::BlockHeader;

        let mut state = 0xA0_52_4F_52_41u64;
        for case in 0..1024usize {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let length = (state as usize ^ case) % 4097;
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                bytes.push(state as u8);
            }

            let _ = AccountBlock::from_canonical_bytes(&bytes);
            let _ = AccountTransaction::from_canonical_bytes(&bytes);
            let _ = BlockHeader::from_canonical_bytes(&bytes);
            let _ = read_blocks(&bytes);
            let _ = decode_peer_list(&bytes);
        }
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
            handle_peer_shared(
                stream,
                &Mutex::new(chain()),
                &Mutex::new(Vec::new()),
                &server_dir,
                &[],
            )
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

#[cfg(test)]
mod handshake_admission_tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn pending() -> (PendingHandshake, TcpStream, PeerCounts) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (stream, remote) = listener.accept().unwrap();
        let counts = Arc::new(Mutex::new(HashMap::new()));
        let lease = PeerLease::acquire(remote.ip(), &counts).unwrap().unwrap();
        (
            PendingHandshake::new(stream, lease).unwrap(),
            client,
            counts,
        )
    }

    fn poll_until_ready(peer: &mut PendingHandshake) -> Result<Vec<u8>, P2pError> {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if let Some(hello) = peer.poll_at(Instant::now())? {
                return Ok(hello);
            }
            assert!(
                Instant::now() < deadline,
                "local test handshake did not arrive"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn fragmented_preface_does_not_consume_the_next_frame() {
        let (mut peer, mut client, counts) = pending();
        client.write_all(&[0, 0]).unwrap();
        assert!(peer.poll_at(Instant::now()).unwrap().is_none());
        client
            .write_all(&[0, 1, WALLET_HELLO, 0, 0, 0, 1, GET_PEERS])
            .unwrap();
        assert_eq!(poll_until_ready(&mut peer).unwrap(), vec![WALLET_HELLO]);
        peer.stream.set_nonblocking(false).unwrap();
        set_timeouts(&peer.stream).unwrap();
        assert_eq!(read_frame(&mut peer.stream).unwrap(), vec![GET_PEERS]);
        drop(peer);
        assert!(counts.lock().unwrap().is_empty());
    }

    #[test]
    fn silent_and_partial_handshakes_expire_without_extending_the_deadline() {
        for prefix in [vec![], vec![0, 0], vec![0, 0, 0, 73, HELLO]] {
            let (mut peer, mut client, counts) = pending();
            client.write_all(&prefix).unwrap();
            assert!(peer.poll_at(Instant::now()).unwrap().is_none());
            let expired = peer.accepted + HANDSHAKE_TIMEOUT;
            assert!(matches!(
                peer.poll_at(expired),
                Err(P2pError::Protocol("handshake deadline exceeded"))
            ));
            drop(peer);
            assert!(counts.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn invalid_prefaces_are_rejected_without_allocating_the_advertised_payload() {
        for bytes in [
            vec![0, 0, 0, 0],
            vec![0, 64, 0, 0],
            vec![255, 255, 255, 255],
            vec![0, 0, 0, 1, HELLO],
        ] {
            let (mut peer, mut client, counts) = pending();
            client.write_all(&bytes).unwrap();
            assert!(matches!(
                poll_until_ready(&mut peer),
                Err(P2pError::Protocol(_))
            ));
            assert_eq!(peer.bytes.len(), 77);
            drop(peer);
            assert!(counts.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn peer_hello_passes_the_preface_gate_without_skipping_consensus_checks() {
        let (mut peer, mut client, _) = pending();
        let mut hello = vec![HELLO];
        hello.extend_from_slice(&[0; 72]);
        write_frame(&mut client, &hello).unwrap();
        let received = poll_until_ready(&mut peer).unwrap();
        assert_eq!(received, hello);
        let genesis = aurora_consensus::testnet::testnet_genesis_block();
        let chain = AccountForkChain::new(
            genesis,
            aurora_consensus::testnet::TESTNET_INITIAL_BITS,
            aurora_core::state::AccountLedger::new(NetworkId::Testnet),
        )
        .unwrap();
        assert!(check_hello(&received, &chain).is_err());
    }
}

#[cfg(test)]
mod self_transfer_tests {
    use super::*;
    use aurora_core::{
        accounts::{AccountTxError, ML_DSA_65_PUBLIC_KEY_BYTES, POST_QUANTUM_AUTH_BYTES},
        amount::Amount,
        signature::{post_quantum_account_id, verify_account_transaction},
        state::{AccountLedger, StateError},
    };
    use ml_dsa::{Keypair, MlDsa65, Signer, SigningKey};

    fn signed_transfer(nonce: u64, to_self: bool) -> AccountTransaction {
        // Public deterministic seed for tests only.
        let key = SigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
        let public = key.verifying_key().encode();
        let sender = post_quantum_account_id(public.as_slice());
        let mut tx = AccountTransaction {
            version: POST_QUANTUM_ACCOUNT_TX_VERSION,
            network: NetworkId::Testnet,
            sender,
            nonce,
            recipient: if to_self {
                sender
            } else {
                AccountId([0x33; 32])
            },
            amount: Amount::from_units(25).unwrap(),
            fee: Amount::from_units(2).unwrap(),
            auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
        };
        tx.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public.as_slice());
        let signature = key.sign(tx.signing_digest().unwrap().as_bytes()).encode();
        tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(signature.as_slice());
        assert_eq!(verify_account_transaction(&tx), Ok(()));
        tx
    }

    fn funded_ledger() -> AccountLedger {
        let mut ledger = AccountLedger::new(NetworkId::Testnet);
        ledger
            .credit_issuance(
                signed_transfer(0, false).sender,
                Amount::from_units(1_000).unwrap(),
            )
            .unwrap();
        ledger
    }

    #[test]
    fn signed_v3_self_transfer_preserves_ledger_and_batch_atomically() {
        let invalid = signed_transfer(0, true);
        assert_eq!(
            invalid.check_stateless(),
            Err(AccountTxError::SameSenderAndRecipient)
        );
        let mut ledger = funded_ledger();
        let before = ledger.clone();
        assert_eq!(
            ledger.apply_transaction(&invalid),
            Err(StateError::InvalidTransaction)
        );
        // Full equality includes every balance, nonce and total issuance.
        assert_eq!(ledger, before);
        let valid = signed_transfer(0, false);
        assert_eq!(
            ledger.apply_batch(&[valid.clone(), signed_transfer(1, true)]),
            Err(StateError::InvalidTransaction)
        );
        assert_eq!(ledger, before);
        assert_eq!(ledger.apply_transaction(&valid), Ok(valid.fee));
        assert_eq!(ledger.account(valid.sender).balance.to_units(), 973);
        assert_eq!(ledger.account(valid.sender).next_nonce, 1);
        assert_eq!(ledger.account(valid.recipient).balance.to_units(), 25);
        assert_eq!(ledger.total_issued(), before.total_issued());
    }

    #[test]
    fn signed_v3_self_transfer_rpc_rejection_preserves_pending_payments() {
        let ledger = funded_ledger();
        let before = ledger.clone();
        let genesis = AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 0, vec![]).unwrap();
        let chain = AccountForkChain::new(genesis, 0, ledger).unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let chain = Mutex::new(chain);
            let pool = Mutex::new(Vec::new());
            let result = handle_peer_shared(stream, &chain, &pool, &std::env::temp_dir(), &[]);
            (
                result,
                chain.into_inner().unwrap(),
                pool.into_inner().unwrap(),
            )
        });
        let mut client = TcpStream::connect(address).unwrap();
        set_timeouts(&client).unwrap();
        write_frame(&mut client, &[WALLET_HELLO]).unwrap();
        assert_eq!(read_frame(&mut client).unwrap()[0], WALLET_HELLO_REPLY);

        let submit = |client: &mut TcpStream, tx: &AccountTransaction| {
            let mut request = vec![SUBMIT_TX];
            request.extend_from_slice(&tx.to_canonical_bytes().unwrap());
            write_frame(client, &request).unwrap();
            read_frame(client).unwrap()
        };
        assert_eq!(
            submit(&mut client, &signed_transfer(0, true)),
            vec![TX_REJECTED, 1]
        );
        write_frame(&mut client, &[GET_MEMPOOL, 0, 0, 0, 0]).unwrap();
        assert_eq!(read_frame(&mut client).unwrap(), vec![MEMPOOL_REPLY, 0, 0]);

        let valid = signed_transfer(0, false);
        assert_eq!(submit(&mut client, &valid)[0], TX_ACCEPTED);
        assert_eq!(
            submit(&mut client, &signed_transfer(1, true)),
            vec![TX_REJECTED, 1]
        );
        drop(client);
        let (result, chain, pool) = server.join().unwrap();
        assert!(result.is_ok());
        assert_eq!(chain.state(), &before);
        assert_eq!(pool, vec![valid.clone()]);

        // Revalidation must also remove an invalid entry without consuming its nonce.
        let pool = Mutex::new(vec![signed_transfer(0, true), valid.clone()]);
        revalidate_mempool(&chain, &pool).unwrap();
        assert_eq!(*pool.lock().unwrap(), vec![valid]);
        assert_eq!(chain.state(), &before);
    }
}
