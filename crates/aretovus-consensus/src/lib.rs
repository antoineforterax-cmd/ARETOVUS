//! Consensus rules and experimental testnet implementation for ARETOVUS.
//!
//! This crate contains the account-chain consensus, cumulative-work fork selection,
//! bounded proof-of-work retargeting, reward accounting, and header verification
//! used by the experimental testnet. Consensus and cryptographic code still require
//! independent review. Do not use this network with real funds.
pub mod account_block;
pub mod account_fork_chain;
pub mod block_validation;
pub mod difficulty;
pub mod experimental;
pub mod fork_chain;
pub mod light_header_chain;
pub mod local_chain;
pub mod proof_of_work;
pub mod reward;
pub mod testnet;
