//! `aurora-consensus` : bloc expérimental et premières expériences de preuve de travail.
//!
//! **Ce crate n'implémente toujours pas un consensus complet.** Le helper PoW est un outil de
//! laboratoire : aucune difficulté variable, récompense, règle temporelle, persistance, ni réseau
//! pair-à-pair n'est implémenté. Les profils de blocs UTXO et comptes restent distincts. Pour les
//! comptes, un sélecteur à difficulté fixe conserve l'état par branche; cela ne règle pas le choix
//! par travail cumulé requis pour une difficulté variable. Les décisions restent ouvertes.
//! persistance, ni réseau pair-à-pair n'est implémenté. Un sélecteur de branches en mémoire existe
//! seulement pour difficulté fixe; les décisions restent ouvertes dans `docs/DECISION_REGISTER.md`.
//!
//! Ne pas utiliser avec de vrais fonds.

pub mod account_block;
pub mod account_fork_chain;
pub mod block_validation;
pub mod difficulty;
pub mod light_header_chain;
pub mod experimental;
pub mod fork_chain;
pub mod local_chain;
pub mod proof_of_work;
pub mod reward;
pub mod testnet;
