//! Bounded TCP wallet requests to an AURORA testnet node.
use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use aurora_consensus::experimental::BlockHeader;
use aurora_core::{
    accounts::{AccountId, AccountTransaction},
    amount::Amount,
    codec::{CanonicalDecode, CanonicalEncode},
    hash::Hash256,
    merkle::MerkleProof,
};

const MAX_FRAME: usize = 4 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(10);
const WALLET_HELLO: u8 = 5;
const WALLET_HELLO_REPLY: u8 = 0x85;
const GET_ACCOUNT: u8 = 6;
const ACCOUNT_REPLY: u8 = 0x86;
const SUBMIT_TX: u8 = 7;
const TX_ACCEPTED: u8 = 0x87;
const TX_REJECTED: u8 = 0x89;
const GET_TX_STATUS: u8 = 9;
const TX_STATUS_REPLY: u8 = 0x8a;
const GET_HEADERS: u8 = 10;
const HEADERS_REPLY: u8 = 0x8b;
const GET_TX_PROOF: u8 = 11;
const TX_PROOF_REPLY: u8 = 0x8c;
const MAX_HEADERS_PAGE: u16 = 64;
const MAX_MERKLE_SIBLINGS: u8 = 17;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderPage {
    pub start_height: u64,
    pub tip_height: u64,
    pub headers: Vec<BlockHeader>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionProof {
    pub height: u64,
    pub header: BlockHeader,
    pub proof: MerkleProof,
}

pub(crate) fn get_headers_on_stream(
    stream: &mut TcpStream,
    start_height: u64,
    limit: u16,
) -> Result<HeaderPage, Box<dyn Error>> {
    if limit == 0 || limit > MAX_HEADERS_PAGE {
        return Err("SPV header page limit is outside 1..=64".into());
    }
    let mut request = vec![GET_HEADERS];
    request.extend_from_slice(&start_height.to_be_bytes());
    request.extend_from_slice(&limit.to_be_bytes());
    write_frame(stream, &request)?;
    let reply = read_frame(stream)?;
    if reply.len() < 19 || reply[0] != HEADERS_REPLY {
        return Err("node returned an invalid SPV header page".into());
    }
    let tip_height = u64::from_be_bytes(reply[1..9].try_into()?);
    let returned_start = u64::from_be_bytes(reply[9..17].try_into()?);
    let count = u16::from_be_bytes(reply[17..19].try_into()?) as usize;
    if returned_start != start_height || count > limit as usize {
        return Err("node returned a mismatched SPV header page".into());
    }
    let mut cursor = 19usize;
    let mut headers = Vec::with_capacity(count);
    for _ in 0..count {
        let length_end = cursor.checked_add(2).ok_or("SPV length overflow")?;
        let length = u16::from_be_bytes(
            reply
                .get(cursor..length_end)
                .ok_or("truncated SPV header length")?
                .try_into()?,
        ) as usize;
        cursor = length_end;
        let end = cursor.checked_add(length).ok_or("SPV length overflow")?;
        let bytes = reply.get(cursor..end).ok_or("truncated SPV header")?;
        headers.push(BlockHeader::from_canonical_bytes(bytes)?);
        cursor = end;
    }
    if cursor != reply.len() {
        return Err("SPV header page has trailing bytes".into());
    }
    Ok(HeaderPage {
        start_height,
        tip_height,
        headers,
    })
}

pub(crate) fn get_transaction_proof_on_stream(
    stream: &mut TcpStream,
    txid: Hash256,
) -> Result<Option<TransactionProof>, Box<dyn Error>> {
    let mut request = vec![GET_TX_PROOF];
    request.extend_from_slice(txid.as_bytes());
    write_frame(stream, &request)?;
    let reply = read_frame(stream)?;
    if reply.len() == 2 && reply[0] == TX_PROOF_REPLY && reply[1] == 0 {
        return Ok(None);
    }
    if reply.len() < 2 || reply[0] != TX_PROOF_REPLY || reply[1] != 1 {
        return Err("node returned an invalid transaction proof response".into());
    }
    let mut cursor = 2usize;
    let height_end = cursor.checked_add(8).ok_or("SPV length overflow")?;
    let height = u64::from_be_bytes(reply.get(cursor..height_end).ok_or("truncated proof height")?.try_into()?);
    cursor = height_end;
    let header_length_end = cursor.checked_add(2).ok_or("SPV length overflow")?;
    let header_length = u16::from_be_bytes(reply.get(cursor..header_length_end).ok_or("truncated proof header length")?.try_into()?) as usize;
    cursor = header_length_end;
    let header_end = cursor.checked_add(header_length).ok_or("SPV length overflow")?;
    let header = BlockHeader::from_canonical_bytes(reply.get(cursor..header_end).ok_or("truncated proof header")?)?;
    cursor = header_end;
    let index_end = cursor.checked_add(4).ok_or("SPV length overflow")?;
    let index = u32::from_be_bytes(reply.get(cursor..index_end).ok_or("truncated proof index")?.try_into()?);
    cursor = index_end;
    let count_end = cursor.checked_add(4).ok_or("SPV length overflow")?;
    let leaf_count = u32::from_be_bytes(reply.get(cursor..count_end).ok_or("truncated proof leaf count")?.try_into()?);
    cursor = count_end;
    let sibling_count = *reply.get(cursor).ok_or("truncated proof sibling count")?;
    cursor += 1;
    if sibling_count > MAX_MERKLE_SIBLINGS {
        return Err("node returned an oversized Merkle proof".into());
    }
    let expected_end = cursor
        .checked_add(usize::from(sibling_count) * 32)
        .ok_or("SPV proof length overflow")?;
    if expected_end != reply.len() {
        return Err("SPV transaction proof has an invalid length".into());
    }
    let mut siblings = Vec::with_capacity(usize::from(sibling_count));
    while cursor < expected_end {
        let end = cursor + 32;
        siblings.push(Hash256::from_bytes(reply[cursor..end].try_into()?));
        cursor = end;
    }
    Ok(Some(TransactionProof {
        height,
        header,
        proof: MerkleProof {
            index,
            leaf_count,
            siblings,
        },
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RpcAccount {
    pub next_nonce: u64,
    pub balance: Amount,
}

fn connect(node: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
    let mut stream = TcpStream::connect_timeout(&node, IO_TIMEOUT)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    write_frame(&mut stream, &[WALLET_HELLO])?;
    let hello = read_frame(&mut stream)?;
    if hello.len() != 74 || hello[0] != WALLET_HELLO_REPLY || hello[1] != 2 {
        return Err("node rejected wallet RPC or is not the AURORA testnet".into());
    }
    Ok(stream)
}

fn write_frame(stream: &mut TcpStream, frame: &[u8]) -> Result<(), Box<dyn Error>> {
    if frame.is_empty() || frame.len() > MAX_FRAME {
        return Err("wallet RPC request exceeds frame limit".into());
    }
    stream.write_all(&(frame.len() as u32).to_be_bytes())?;
    stream.write_all(frame)?;
    stream.flush()?;
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut length = [0; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME {
        return Err("wallet RPC response exceeds frame limit".into());
    }
    let mut frame = vec![0; length];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

pub fn get_account(node: SocketAddr, id: AccountId) -> Result<RpcAccount, Box<dyn Error>> {
    let mut stream = connect(node)?;
    let mut request = vec![GET_ACCOUNT];
    request.extend_from_slice(&id.0);
    write_frame(&mut stream, &request)?;
    let reply = read_frame(&mut stream)?;
    if reply.len() != 17 || reply[0] != ACCOUNT_REPLY {
        return Err("node returned an invalid account response".into());
    }
    let next_nonce = u64::from_be_bytes(reply[1..9].try_into()?);
    let balance = u64::from_be_bytes(reply[9..17].try_into()?);
    Ok(RpcAccount {
        next_nonce,
        balance: Amount::from_units(balance)?,
    })
}

pub fn submit_transaction(
    node: SocketAddr,
    tx: &AccountTransaction,
) -> Result<(Hash256, u32), Box<dyn Error>> {
    let mut stream = connect(node)?;
    let mut request = vec![SUBMIT_TX];
    request.extend_from_slice(&tx.to_canonical_bytes()?);
    write_frame(&mut stream, &request)?;
    let reply = read_frame(&mut stream)?;
    match reply.first() {
        Some(&TX_ACCEPTED) if reply.len() == 37 => {
            let txid = Hash256::from_bytes(reply[1..33].try_into()?);
            let pending_count = u32::from_be_bytes(reply[33..37].try_into()?);
            Ok((txid, pending_count))
        }
        Some(&TX_REJECTED) if reply.len() == 2 => {
            let reason = match reply[1] {
                1 => "wrong network, version, or transaction encoding",
                2 => "invalid signature, nonce, or insufficient balance",
                3 => "mempool capacity reached",
                _ => "unknown rejection",
            };
            Err(format!("node rejected transaction: {reason}").into())
        }
        _ => Err("node returned an invalid transaction response".into()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionStatus {
    Unknown,
    Pending,
    Confirmed,
}

pub fn transaction_status(
    node: SocketAddr,
    txid: Hash256,
) -> Result<TransactionStatus, Box<dyn Error>> {
    let mut stream = connect(node)?;
    let mut request = vec![GET_TX_STATUS];
    request.extend_from_slice(txid.as_bytes());
    write_frame(&mut stream, &request)?;
    let reply = read_frame(&mut stream)?;
    if reply.len() != 2 || reply[0] != TX_STATUS_REPLY {
        return Err("node returned an invalid transaction status".into());
    }
    match reply[1] {
        0 => Ok(TransactionStatus::Unknown),
        1 => Ok(TransactionStatus::Pending),
        2 => Ok(TransactionStatus::Confirmed),
        _ => Err("node returned an unknown transaction status".into()),
    }
}
