//! Bounded TCP wallet requests to an AURORA testnet node.
use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use aurora_core::{
    accounts::{AccountId, AccountTransaction},
    amount::Amount,
    codec::CanonicalEncode,
    hash::Hash256,
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
