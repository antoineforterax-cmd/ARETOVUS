//! Registre de comptes déterministe pour le prototype ARETOVUS.
//!
//! Testnet exige ML-DSA-65 natif (profil v3); Regtest conserve les profils historiques.
//! Les transferts passent la vérification de signature du profil versionné.
//! Aucun transfert ne modifie l'état si cette vérification échoue.
//! Ce module ne constitue pas encore une validation complète de bloc ou de consensus.

use std::collections::BTreeMap;
use std::fmt;

use crate::accounts::{AccountId, AccountTransaction};
use crate::amount::{Amount, AmountError};
use crate::network::NetworkId;
use crate::signature::verify_account_transaction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccountState {
    pub balance: Amount,
    pub next_nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountLedger {
    network: NetworkId,
    accounts: BTreeMap<AccountId, AccountState>,
    /// Cumul des unités émises par genèse/récompenses, indépendant des transferts.
    issued: Amount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    InvalidTransaction,
    WrongNetwork,
    InvalidAuthorization,
    UnknownSender,
    NonceMismatch { expected: u64, actual: u64 },
    InsufficientFunds,
    BalanceOutOfRange(AmountError),
    NonceExhausted,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => write!(f, "transaction invalide sans état"),
            Self::WrongNetwork => write!(f, "transaction destinée à un autre réseau"),
            Self::InvalidAuthorization => write!(f, "autorisation invalide"),
            Self::UnknownSender => write!(f, "compte expéditeur inconnu ou non financé"),
            Self::NonceMismatch { expected, actual } => {
                write!(f, "nonce incorrect : attendu {expected}, reçu {actual}")
            }
            Self::InsufficientFunds => write!(f, "solde insuffisant"),
            Self::BalanceOutOfRange(err) => write!(f, "solde hors limites : {err}"),
            Self::NonceExhausted => write!(f, "nonce épuisé"),
        }
    }
}

impl std::error::Error for StateError {}

impl AccountLedger {
    pub fn new(network: NetworkId) -> Self {
        Self {
            network,
            accounts: BTreeMap::new(),
            issued: Amount::ZERO,
        }
    }

    pub fn account(&self, id: AccountId) -> AccountState {
        self.accounts.get(&id).copied().unwrap_or_default()
    }

    pub fn network(&self) -> NetworkId {
        self.network
    }

    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Total cumulatif émis. Les frais brûlés ne sont pas encore traités par ce registre.
    pub fn total_issued(&self) -> Amount {
        self.issued
    }

    /// Crédit réservé à l'émission/genèse contrôlée par le nœud.
    /// Ne jamais exposer directement cette méthode à une API réseau publique.
    pub fn credit_issuance(
        &mut self,
        recipient: AccountId,
        amount: Amount,
    ) -> Result<(), StateError> {
        if amount == Amount::ZERO {
            return Ok(());
        }
        let next_issued = self
            .issued
            .checked_add(amount)
            .map_err(StateError::BalanceOutOfRange)?;
        let current = self.account(recipient);
        let balance = current
            .balance
            .checked_add(amount)
            .map_err(StateError::BalanceOutOfRange)?;
        // Mettre à jour l'état seulement après validation des deux sommes.
        self.accounts
            .insert(recipient, AccountState { balance, ..current });
        self.issued = next_issued;
        Ok(())
    }

    /// Credits the miner's share of transaction fees without increasing gross issuance.
    /// Intended for the consensus block-reward validator, never a network-facing API.
    pub fn credit_fee_reward(
        &mut self,
        recipient: AccountId,
        amount: Amount,
    ) -> Result<(), StateError> {
        if amount == Amount::ZERO {
            return Ok(());
        }
        let current = self.account(recipient);
        let balance = current
            .balance
            .checked_add(amount)
            .map_err(StateError::BalanceOutOfRange)?;
        self.accounts
            .insert(recipient, AccountState { balance, ..current });
        Ok(())
    }

    /// Applique un transfert atomiquement après les contrôles d'état et d'autorisation.
    /// La signature est vérifiée avant toute mutation.
    pub fn apply_transaction(&mut self, tx: &AccountTransaction) -> Result<Amount, StateError> {
        self.apply_transaction_verified(tx, verify_account_transaction)
    }

    /// Rechecks all state rules, reusing only successful cryptographic checks.
    pub fn apply_transaction_cached(
        &mut self,
        tx: &AccountTransaction,
    ) -> Result<Amount, StateError> {
        self.apply_transaction_verified(tx, crate::signature::verify_account_transaction_cached)
    }

    fn apply_transaction_verified(
        &mut self,
        tx: &AccountTransaction,
        verify: fn(&AccountTransaction) -> Result<(), crate::signature::SignatureError>,
    ) -> Result<Amount, StateError> {
        tx.check_stateless()
            .map_err(|_| StateError::InvalidTransaction)?;
        if tx.network != self.network {
            return Err(StateError::WrongNetwork);
        }

        let sender_state = self
            .accounts
            .get(&tx.sender)
            .copied()
            .ok_or(StateError::UnknownSender)?;

        if tx.nonce != sender_state.next_nonce {
            return Err(StateError::NonceMismatch {
                expected: sender_state.next_nonce,
                actual: tx.nonce,
            });
        }

        if verify(tx).is_err() {
            return Err(StateError::InvalidAuthorization);
        }

        let total_debit = tx
            .amount
            .checked_add(tx.fee)
            .map_err(StateError::BalanceOutOfRange)?;
        let sender_balance = sender_state
            .balance
            .checked_sub(total_debit)
            .map_err(|_| StateError::InsufficientFunds)?;
        let next_nonce = sender_state
            .next_nonce
            .checked_add(1)
            .ok_or(StateError::NonceExhausted)?;

        let recipient_state = self.account(tx.recipient);
        let recipient_balance = recipient_state
            .balance
            .checked_add(tx.amount)
            .map_err(StateError::BalanceOutOfRange)?;

        // Toutes les opérations susceptibles d'échouer précèdent les écritures.
        self.accounts.insert(
            tx.sender,
            AccountState {
                balance: sender_balance,
                next_nonce,
            },
        );
        self.accounts.insert(
            tx.recipient,
            AccountState {
                balance: recipient_balance,
                ..recipient_state
            },
        );
        // Le validateur de bloc doit comptabiliser ces frais puis appliquer la règle de partage.
        Ok(tx.fee)
    }

    /// Applique un lot dans l'ordre, sans mutation partielle si une transaction échoue.
    /// Retourne la somme des frais pour le validateur de bloc appelant.
    pub fn apply_batch(
        &mut self,
        transactions: &[AccountTransaction],
    ) -> Result<Amount, StateError> {
        let mut candidate = self.clone();
        let mut total_fees = Amount::ZERO;
        for tx in transactions {
            let fee = candidate.apply_transaction(tx)?;
            total_fees = total_fees
                .checked_add(fee)
                .map_err(StateError::BalanceOutOfRange)?;
        }
        *self = candidate;
        Ok(total_fees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{AccountTransaction, ACCOUNT_TX_VERSION};
    use crate::network::NetworkId;
    use ed25519_dalek::{Signer, SigningKey};

    fn id(byte: u8) -> AccountId {
        AccountId([byte; 32])
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[1; 32])
    }

    fn tx(nonce: u64, amount: u64, fee: u64) -> AccountTransaction {
        let key = signing_key();
        let mut transaction = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId(key.verifying_key().to_bytes()),
            nonce,
            recipient: id(2),
            amount: Amount::from_units(amount).unwrap(),
            fee: Amount::from_units(fee).unwrap(),
            auth_data: Vec::new(),
        };
        transaction.auth_data = key
            .sign(transaction.signing_digest().unwrap().as_bytes())
            .to_bytes()
            .to_vec();
        transaction
    }

    fn funded_ledger() -> AccountLedger {
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        ledger
            .credit_issuance(
                AccountId(signing_key().verifying_key().to_bytes()),
                Amount::from_units(1_000).unwrap(),
            )
            .unwrap();
        ledger
    }

    #[test]
    fn valid_transfer_debits_amount_and_fee_and_increments_nonce() {
        let mut ledger = funded_ledger();
        ledger.apply_transaction(&tx(0, 200, 5)).unwrap();
        let sender = tx(0, 200, 5).sender;
        assert_eq!(ledger.account(sender).balance.to_units(), 795);
        assert_eq!(ledger.account(sender).next_nonce, 1);
        assert_eq!(ledger.account(id(2)).balance.to_units(), 200);
        assert_eq!(ledger.account(id(2)).next_nonce, 0);
    }

    #[test]
    fn wrong_network_is_rejected_without_mutation() {
        let mut ledger = funded_ledger();
        let before = ledger.clone();
        let mut transaction = tx(0, 10, 1);
        // Use the native testnet transaction shape: an obsolete v1 profile
        // must fail stateless validation before the network mismatch branch.
        transaction.network = NetworkId::Testnet;
        transaction.version = crate::accounts::POST_QUANTUM_ACCOUNT_TX_VERSION;
        transaction.auth_data = vec![0; crate::accounts::POST_QUANTUM_AUTH_BYTES];
        assert_eq!(
            ledger.apply_transaction(&transaction),
            Err(StateError::WrongNetwork)
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn invalid_authorization_leaves_state_unchanged() {
        let mut ledger = funded_ledger();
        let before = ledger.clone();
        let mut invalid = tx(0, 200, 5);
        invalid.auth_data[0] ^= 1;
        assert_eq!(
            ledger.apply_transaction(&invalid),
            Err(StateError::InvalidAuthorization)
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn insufficient_funds_leaves_state_unchanged() {
        let mut ledger = funded_ledger();
        let before = ledger.clone();
        assert_eq!(
            ledger.apply_transaction(&tx(0, 999, 2)),
            Err(StateError::InsufficientFunds)
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn replayed_or_skipped_nonce_is_rejected() {
        let mut ledger = funded_ledger();
        ledger.apply_transaction(&tx(0, 10, 1)).unwrap();
        assert_eq!(
            ledger.apply_transaction(&tx(0, 10, 1)),
            Err(StateError::NonceMismatch {
                expected: 1,
                actual: 0
            })
        );
        assert_eq!(
            ledger.apply_transaction(&tx(2, 10, 1)),
            Err(StateError::NonceMismatch {
                expected: 1,
                actual: 2
            })
        );
    }

    #[test]
    fn missing_sender_and_global_issuance_overflow_are_rejected_atomically() {
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        assert_eq!(
            ledger.apply_transaction(&tx(0, 1, 0)),
            Err(StateError::UnknownSender)
        );

        ledger
            .credit_issuance(
                AccountId(signing_key().verifying_key().to_bytes()),
                Amount::MAX,
            )
            .unwrap();
        let before = ledger.clone();
        assert_eq!(
            ledger.credit_issuance(id(2), Amount::from_units(1).unwrap()),
            Err(StateError::BalanceOutOfRange(AmountError::AboveCap))
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn issuance_credit_is_bounded_per_account_and_globally() {
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        ledger.credit_issuance(id(1), Amount::MAX).unwrap();
        assert_eq!(ledger.total_issued(), Amount::MAX);
        let before = ledger.clone();
        assert!(matches!(
            ledger.credit_issuance(id(1), Amount::from_units(1).unwrap()),
            Err(StateError::BalanceOutOfRange(_))
        ));
        assert_eq!(ledger, before);

        // Le plafond s'applique à l'offre totale, pas seulement à chaque compte.
        assert!(matches!(
            ledger.credit_issuance(id(2), Amount::from_units(1).unwrap()),
            Err(StateError::BalanceOutOfRange(_))
        ));
        assert_eq!(ledger, before);
    }

    #[test]
    fn batch_applies_in_order_and_returns_total_fees() {
        let mut ledger = funded_ledger();
        let transactions = [tx(0, 20, 2), tx(1, 30, 3)];
        assert_eq!(ledger.apply_batch(&transactions).unwrap().to_units(), 5);
        let sender = transactions[0].sender;
        assert_eq!(ledger.account(sender).balance.to_units(), 945);
        assert_eq!(ledger.account(sender).next_nonce, 2);
        assert_eq!(ledger.account(id(2)).balance.to_units(), 50);
    }

    #[test]
    fn failed_batch_does_not_apply_earlier_transactions() {
        let mut ledger = funded_ledger();
        let before = ledger.clone();
        let transactions = [tx(0, 20, 2), tx(2, 30, 3)];
        assert!(matches!(
            ledger.apply_batch(&transactions),
            Err(StateError::NonceMismatch {
                expected: 1,
                actual: 2
            })
        ));
        assert_eq!(ledger, before);
    }
}
