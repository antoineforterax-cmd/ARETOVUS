//! Validation expérimentale de blocs de transferts signés par comptes.
use std::{collections::HashSet, fmt};

use crate::{
    experimental::{BlockHeader, HEADER_VERSION, MAX_BLOCK_TXS},
    proof_of_work::{check_proof, PowError},
};
use aurora_core::{
    accounts::{AccountId, AccountTransaction, AccountTxError},
    amount::Amount,
    codec::{
        decode_vec, encode_vec, CanonicalDecode, CanonicalEncode, DecodeError, Decoder,
        EncodeError, Encoder,
    },
    hash::{tagged_hash, Domain, Hash256},
    network::NetworkId,
    state::{AccountLedger, StateError},
};

pub const MAX_ACCOUNT_BLOCK_BYTES: usize = 4 * 1024 * 1024 - 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountBlock {
    pub header: BlockHeader,
    pub transactions: Vec<AccountTransaction>,
    /// Miner payout identity, committed by `header.tx_commitment`.
    pub miner: Option<AccountId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountBlockError {
    UnsupportedHeaderVersion,
    TooManyTransactions,
    NetworkMismatch,
    InvalidTransaction(AccountTxError),
    DuplicateTransaction,
    CommitmentMismatch,
    BlockTooLarge,
    Encoding(EncodeError),
}

impl fmt::Display for AccountBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AccountBlockError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountBlockValidationError {
    Stateless(AccountBlockError),
    State(StateError),
    ProofOfWork(PowError),
    InsufficientWork,
}
impl fmt::Display for AccountBlockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AccountBlockValidationError {}

pub fn compute_account_tx_commitment(txs: &[AccountTransaction]) -> Result<Hash256, EncodeError> {
    compute_account_block_commitment(txs, None)
}

/// Commits both ordered transaction IDs and the miner identity into the PoW header.
pub fn compute_account_block_commitment(
    txs: &[AccountTransaction],
    miner: Option<AccountId>,
) -> Result<Hash256, EncodeError> {
    let mut enc = Encoder::new();
    enc.put_len(txs.len(), MAX_BLOCK_TXS)?;
    for tx in txs {
        tx.txid()?.encode(&mut enc)?;
    }
    match miner {
        None => enc.put_u8(0),
        Some(account) => {
            enc.put_u8(1);
            account.encode(&mut enc)?;
        }
    }
    Ok(tagged_hash(Domain::AccountTxList, &enc.into_bytes()))
}

impl AccountBlock {
    pub fn build(
        network: NetworkId,
        prev_hash: Hash256,
        timestamp: u64,
        nonce: u64,
        transactions: Vec<AccountTransaction>,
    ) -> Result<Self, EncodeError> {
        Self::build_with_miner(network, prev_hash, timestamp, nonce, None, transactions)
    }

    pub fn build_with_miner(
        network: NetworkId,
        prev_hash: Hash256,
        timestamp: u64,
        nonce: u64,
        miner: Option<AccountId>,
        transactions: Vec<AccountTransaction>,
    ) -> Result<Self, EncodeError> {
        let tx_commitment = compute_account_block_commitment(&transactions, miner)?;
        Ok(Self {
            header: BlockHeader {
                version: HEADER_VERSION,
                network,
                prev_hash,
                tx_commitment,
                timestamp,
                nonce,
            },
            transactions,
            miner,
        })
    }

    pub fn check_stateless(&self) -> Result<(), AccountBlockError> {
        if self.header.version != HEADER_VERSION {
            return Err(AccountBlockError::UnsupportedHeaderVersion);
        }
        if self.transactions.len() > MAX_BLOCK_TXS {
            return Err(AccountBlockError::TooManyTransactions);
        }
        let mut seen = HashSet::with_capacity(self.transactions.len());
        for tx in &self.transactions {
            if tx.network != self.header.network {
                return Err(AccountBlockError::NetworkMismatch);
            }
            tx.check_stateless()
                .map_err(AccountBlockError::InvalidTransaction)?;
            if !seen.insert(tx.txid().map_err(AccountBlockError::Encoding)?) {
                return Err(AccountBlockError::DuplicateTransaction);
            }
        }
        if compute_account_block_commitment(&self.transactions, self.miner)
            .map_err(AccountBlockError::Encoding)?
            != self.header.tx_commitment
        {
            return Err(AccountBlockError::CommitmentMismatch);
        }
        if self
            .to_canonical_bytes()
            .map_err(AccountBlockError::Encoding)?
            .len()
            > MAX_ACCOUNT_BLOCK_BYTES
        {
            return Err(AccountBlockError::BlockTooLarge);
        }
        Ok(())
    }
}

pub fn check_account_block_pow(
    block: &AccountBlock,
    difficulty_bits: u16,
) -> Result<(), AccountBlockValidationError> {
    block
        .check_stateless()
        .map_err(AccountBlockValidationError::Stateless)?;
    match check_proof(&block.header, difficulty_bits)
        .map_err(AccountBlockValidationError::ProofOfWork)?
    {
        true => Ok(()),
        false => Err(AccountBlockValidationError::InsufficientWork),
    }
}

/// Valide puis applique tout le lot; les frais sont retournés mais aucune récompense n'est créée.
pub fn validate_and_apply_account_block(
    block: &AccountBlock,
    difficulty_bits: u16,
    ledger: &mut AccountLedger,
) -> Result<Amount, AccountBlockValidationError> {
    check_account_block_pow(block, difficulty_bits)?;
    if block.header.network != ledger.network() {
        return Err(AccountBlockValidationError::State(StateError::WrongNetwork));
    }
    ledger
        .apply_batch(&block.transactions)
        .map_err(AccountBlockValidationError::State)
}

impl CanonicalEncode for AccountBlock {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        self.header.encode(enc)?;
        match self.miner {
            None => enc.put_u8(0),
            Some(account) => {
                enc.put_u8(1);
                account.encode(enc)?;
            }
        }
        encode_vec(&self.transactions, MAX_BLOCK_TXS, enc)
    }
}
impl CanonicalDecode for AccountBlock {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        if dec.remaining() > MAX_ACCOUNT_BLOCK_BYTES {
            return Err(DecodeError::LengthTooLarge {
                len: dec.remaining() as u64,
                max: MAX_ACCOUNT_BLOCK_BYTES,
            });
        }
        let header = BlockHeader::decode(dec)?;
        let miner = match dec.get_u8()? {
            0 => None,
            1 => Some(AccountId::decode(dec)?),
            _ => return Err(DecodeError::InvalidValue("invalid miner option tag")),
        };
        Ok(Self {
            header,
            transactions: decode_vec(dec, MAX_BLOCK_TXS)?,
            miner,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::{
        accounts::{AccountId, ACCOUNT_TX_VERSION},
        network::NetworkId,
    };
    use ed25519_dalek::{Signer, SigningKey};

    fn account(byte: u8) -> AccountId {
        AccountId([byte; 32])
    }
    fn key() -> SigningKey {
        SigningKey::from_bytes(&[4; 32])
    }
    fn signed_tx(nonce: u64, recipient: u8, amount: u64, fee: u64) -> AccountTransaction {
        let signing_key = key();
        let mut tx = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId(signing_key.verifying_key().to_bytes()),
            nonce,
            recipient: account(recipient),
            amount: Amount::from_units(amount).unwrap(),
            fee: Amount::from_units(fee).unwrap(),
            auth_data: Vec::new(),
        };
        tx.auth_data = signing_key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .to_bytes()
            .to_vec();
        tx
    }
    fn funded() -> AccountLedger {
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        ledger
            .credit_issuance(
                AccountId(key().verifying_key().to_bytes()),
                Amount::from_units(1_000).unwrap(),
            )
            .unwrap();
        ledger
    }

    #[test]
    fn valid_block_roundtrips_and_applies_signed_transfers() {
        let mut ledger = funded();
        let txs = vec![signed_tx(0, 8, 20, 2), signed_tx(1, 9, 30, 3)];
        let block = AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 10, 0, txs).unwrap();
        let bytes = block.to_canonical_bytes().unwrap();
        assert_eq!(AccountBlock::from_canonical_bytes(&bytes).unwrap(), block);
        assert_eq!(
            validate_and_apply_account_block(&block, 0, &mut ledger)
                .unwrap()
                .to_units(),
            5
        );
        assert_eq!(ledger.account(account(8)).balance.to_units(), 20);
        assert_eq!(ledger.account(account(9)).balance.to_units(), 30);
        assert_eq!(ledger.total_issued().to_units(), 1_000);
    }

    #[test]
    fn oversized_account_blocks_are_rejected_by_validation_and_decode() {
        let mut txs = Vec::with_capacity(1100);
        for nonce in 0..1100 {
            txs.push(AccountTransaction {
                version: ACCOUNT_TX_VERSION,
                network: NetworkId::Regtest,
                sender: account(1),
                nonce,
                recipient: account(2),
                amount: Amount::from_units(1).unwrap(),
                fee: Amount::ZERO,
                auth_data: vec![0; 4096],
            });
        }
        let block = AccountBlock::build(NetworkId::Regtest, Hash256::ZERO, 1, 0, txs).unwrap();
        assert!(matches!(
            block.check_stateless(),
            Err(AccountBlockError::BlockTooLarge)
        ));
        let encoded = block.to_canonical_bytes().unwrap();
        assert!(AccountBlock::from_canonical_bytes(&encoded).is_err());
    }

    #[test]
    fn miner_identity_is_committed_by_the_pow_header() {
        let mut block = AccountBlock::build_with_miner(
            NetworkId::Regtest,
            Hash256::ZERO,
            10,
            0,
            Some(account(7)),
            Vec::new(),
        )
        .unwrap();
        block.miner = Some(account(8));
        assert_eq!(
            block.check_stateless(),
            Err(AccountBlockError::CommitmentMismatch)
        );
    }

    #[test]
    fn invalid_later_transaction_rolls_back_whole_block() {
        let mut ledger = funded();
        let before = ledger.clone();
        let mut invalid = signed_tx(1, 9, 30, 3);
        invalid.auth_data[0] ^= 1;
        let block = AccountBlock::build(
            NetworkId::Regtest,
            Hash256::ZERO,
            10,
            0,
            vec![signed_tx(0, 8, 20, 2), invalid],
        )
        .unwrap();
        assert_eq!(
            validate_and_apply_account_block(&block, 0, &mut ledger),
            Err(AccountBlockValidationError::State(
                StateError::InvalidAuthorization
            ))
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn empty_block_rejects_wrong_ledger_network() {
        let block = AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 0, vec![]).unwrap();
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        assert_eq!(
            validate_and_apply_account_block(&block, 0, &mut ledger),
            Err(AccountBlockValidationError::State(StateError::WrongNetwork))
        );
    }
}
