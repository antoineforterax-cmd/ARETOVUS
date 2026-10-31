//! Bounded TCP wallet requests to an ARETOVUS testnet node.
use std::{
    error::Error,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};

use aretovus_consensus::{experimental::BlockHeader, testnet::testnet_protocol_fingerprint};
use aretovus_core::{
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
    let height = u64::from_be_bytes(
        reply
            .get(cursor..height_end)
            .ok_or("truncated proof height")?
            .try_into()?,
    );
    cursor = height_end;
    let header_length_end = cursor.checked_add(2).ok_or("SPV length overflow")?;
    let header_length = u16::from_be_bytes(
        reply
            .get(cursor..header_length_end)
            .ok_or("truncated proof header length")?
            .try_into()?,
    ) as usize;
    cursor = header_length_end;
    let header_end = cursor
        .checked_add(header_length)
        .ok_or("SPV length overflow")?;
    let header = BlockHeader::from_canonical_bytes(
        reply
            .get(cursor..header_end)
            .ok_or("truncated proof header")?,
    )?;
    cursor = header_end;
    let index_end = cursor.checked_add(4).ok_or("SPV length overflow")?;
    let index = u32::from_be_bytes(
        reply
            .get(cursor..index_end)
            .ok_or("truncated proof index")?
            .try_into()?,
    );
    cursor = index_end;
    let count_end = cursor.checked_add(4).ok_or("SPV length overflow")?;
    let leaf_count = u32::from_be_bytes(
        reply
            .get(cursor..count_end)
            .ok_or("truncated proof leaf count")?
            .try_into()?,
    );
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

pub(super) fn connect(node: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
    let mut stream = TcpStream::connect_timeout(&node, IO_TIMEOUT)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    write_frame(&mut stream, &[WALLET_HELLO])?;
    let hello = read_frame(&mut stream)?;
    if hello.len() != 74 || hello[0] != WALLET_HELLO_REPLY || hello[1] != 2 {
        return Err("node rejected wallet RPC or is not the ARETOVUS testnet".into());
    }
    if &hello[2..34] != testnet_protocol_fingerprint().as_bytes() {
        return Err("node uses different testnet consensus or peer protocol rules".into());
    }
    Ok(stream)
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

fn write_frame(stream: &mut TcpStream, frame: &[u8]) -> Result<(), Box<dyn Error>> {
    if frame.is_empty() || frame.len() > MAX_FRAME {
        return Err("wallet RPC request exceeds frame limit".into());
    }
    let original = stream.write_timeout()?;
    let budget = original.unwrap_or(IO_TIMEOUT).min(IO_TIMEOUT);
    let started = Instant::now();
    let result = (|| -> io::Result<()> {
        write_all_before_deadline(stream, &(frame.len() as u32).to_be_bytes(), started, budget)?;
        write_all_before_deadline(stream, frame, started, budget)
    })();
    stream.set_write_timeout(original)?;
    Ok(result?)
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let original = stream.read_timeout()?;
    let budget = original.unwrap_or(IO_TIMEOUT).min(IO_TIMEOUT);
    let started = Instant::now();
    let result: Result<Vec<u8>, Box<dyn Error>> = (|| {
        let mut length = [0; 4];
        read_exact_before_deadline(stream, &mut length, started, budget)?;
        let length = u32::from_be_bytes(length) as usize;
        if length == 0 || length > MAX_FRAME {
            return Err("wallet RPC response exceeds frame limit".into());
        }
        let mut frame = vec![0; length];
        read_exact_before_deadline(stream, &mut frame, started, budget)?;
        Ok(frame)
    })();
    stream.set_read_timeout(original)?;
    result
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
    let encoded = tx.to_canonical_bytes()?;
    let expected =
        aretovus_core::hash::tagged_hash(aretovus_core::hash::Domain::AccountTxId, &encoded);
    let mut stream = connect(node)?;
    let mut request = vec![SUBMIT_TX];
    request.extend_from_slice(&encoded);
    write_frame(&mut stream, &request)?;
    let reply = read_frame(&mut stream)?;
    match reply.first() {
        Some(&TX_ACCEPTED) if reply.len() == 37 => {
            let txid = Hash256::from_bytes(reply[1..33].try_into()?);
            if txid != expected {
                return Err(format!(
                    "node acknowledged a different transaction id; submission outcome unknown; check status for {expected}"
                ).into());
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{Ipv4Addr, TcpListener},
        thread,
    };

    #[test]
    fn wallet_rejects_a_node_with_different_protocol_fingerprint() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            assert_eq!(read_frame(&mut stream).unwrap(), vec![WALLET_HELLO]);
            let mut reply = vec![WALLET_HELLO_REPLY, 2];
            reply.extend_from_slice(&[0; 32]);
            reply.extend_from_slice(&0u64.to_be_bytes());
            reply.extend_from_slice(&[0; 32]);
            write_frame(&mut stream, &reply).unwrap();
        });
        assert!(connect(address).is_err());
        server.join().unwrap();
    }
}

#[cfg(test)]
mod deadline_tests {
    use super::*;
    use std::{net::TcpListener, thread};

    fn pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        (client, server)
    }

    #[test]
    fn trickled_prefix_and_payload_share_one_deadline() {
        let (mut client, mut server) = pair();
        client
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let original = client.read_timeout().unwrap();
        let writer = thread::spawn(move || {
            server.write_all(&[0, 0]).unwrap();
            thread::sleep(Duration::from_millis(100));
            let _ = server.write_all(&[0, 8]);
            for _ in 0..8 {
                thread::sleep(Duration::from_millis(50));
                if server.write_all(&[1]).is_err() {
                    break;
                }
            }
        });
        let started = Instant::now();
        let error = read_frame(&mut client).unwrap_err();
        let error = error.downcast_ref::<io::Error>().unwrap();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ));
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(client.read_timeout().unwrap(), original);
        drop(client);
        writer.join().unwrap();
    }

    #[test]
    fn truncated_or_oversized_frames_restore_read_timeout() {
        for prefix in [
            vec![0, 0],
            0u32.to_be_bytes().to_vec(),
            ((MAX_FRAME + 1) as u32).to_be_bytes().to_vec(),
        ] {
            let (mut client, mut server) = pair();
            client
                .set_read_timeout(Some(Duration::from_millis(300)))
                .unwrap();
            let original = client.read_timeout().unwrap();
            server.write_all(&prefix).unwrap();
            drop(server);
            assert!(read_frame(&mut client).is_err());
            assert_eq!(client.read_timeout().unwrap(), original);
        }
    }

    #[test]
    fn successful_frames_preserve_socket_timeouts() {
        let (mut client, mut server) = pair();
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        server
            .set_write_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let read_original = client.read_timeout().unwrap();
        let write_original = server.write_timeout().unwrap();
        for bytes in [vec![1], vec![2, 3, 4]] {
            write_frame(&mut server, &bytes).unwrap();
            assert_eq!(read_frame(&mut client).unwrap(), bytes);
            assert_eq!(client.read_timeout().unwrap(), read_original);
            assert_eq!(server.write_timeout().unwrap(), write_original);
        }
    }

    #[test]
    fn a_node_that_does_not_read_cannot_hold_a_wallet_write_indefinitely() {
        let (mut client, _server) = pair();
        client
            .set_write_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        let original = client.write_timeout().unwrap();
        let payload = vec![0; MAX_FRAME];
        let started = Instant::now();
        let mut blocked = false;
        for _ in 0..32 {
            match write_frame(&mut client, &payload) {
                Ok(()) => {}
                Err(error) => {
                    let error = error.downcast_ref::<io::Error>().unwrap();
                    assert!(matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ));
                    blocked = true;
                    break;
                }
            }
        }
        assert!(blocked);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(client.write_timeout().unwrap(), original);
    }
}

#[cfg(test)]
mod receipt_tests {
    use super::*;
    use crate::{build_signed_transaction, WalletKey};
    use std::{net::TcpListener, thread};

    fn payment() -> AccountTransaction {
        build_signed_transaction(
            &WalletKey::from_seed_bytes([61; 32]),
            AccountId([62; 32]),
            Amount::from_units(100).unwrap(),
            Amount::from_units(3).unwrap(),
            0,
        )
        .unwrap()
    }

    fn submit_with_receipt(
        tx: &AccountTransaction,
        receipt: Vec<u8>,
    ) -> Result<(Hash256, u32), Box<dyn Error>> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let encoded = tx.to_canonical_bytes().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            assert_eq!(read_frame(&mut stream).unwrap(), vec![WALLET_HELLO]);
            let mut hello = vec![WALLET_HELLO_REPLY, 2];
            hello.extend_from_slice(testnet_protocol_fingerprint().as_bytes());
            hello.extend_from_slice(&0u64.to_be_bytes());
            hello.extend_from_slice(&[0; 32]);
            write_frame(&mut stream, &hello).unwrap();
            let request = read_frame(&mut stream).unwrap();
            assert_eq!(request[0], SUBMIT_TX);
            assert_eq!(&request[1..], encoded);
            write_frame(&mut stream, &receipt).unwrap();
        });
        let result = submit_transaction(address, tx);
        server.join().unwrap();
        result
    }

    fn accepted(id: Hash256) -> Vec<u8> {
        let mut reply = vec![TX_ACCEPTED];
        reply.extend_from_slice(id.as_bytes());
        reply.extend_from_slice(&1u32.to_be_bytes());
        reply
    }

    #[test]
    fn submission_accepts_only_the_exact_prepared_transaction_id() {
        let tx = payment();
        assert_eq!(
            submit_with_receipt(&tx, accepted(tx.txid().unwrap())).unwrap(),
            (tx.txid().unwrap(), 1)
        );
        let error = submit_with_receipt(&tx, accepted(Hash256::ZERO))
            .unwrap_err()
            .to_string();
        assert!(error.contains("different transaction id"));
        assert!(error.contains("outcome unknown"));
        assert!(error.contains(&tx.txid().unwrap().to_string()));
    }

    #[test]
    fn truncated_trailing_and_rejected_receipts_never_report_success() {
        let tx = payment();
        let mut trailing = accepted(tx.txid().unwrap());
        trailing.push(0);
        for reply in [vec![TX_ACCEPTED], trailing, vec![TX_REJECTED, 2]] {
            assert!(submit_with_receipt(&tx, reply).is_err());
        }
    }
}
