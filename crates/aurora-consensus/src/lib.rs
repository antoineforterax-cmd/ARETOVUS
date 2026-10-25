//! `aurora-consensus` : bloc expérimental et premières expériences de preuve de travail.
//!
//! **Ce crate n'implémente toujours pas un consensus complet.** Le helper PoW est un outil de
//! laboratoire : aucune difficulté variable, récompense, règle temporelle, validation d'état,
//! persistance, ni réseau pair-à-pair n'est implémenté. Un sélecteur de branches en mémoire existe
//! seulement pour difficulté fixe; les décisions restent ouvertes dans `docs/DECISION_REGISTER.md`.
//!
//! Ne pas utiliser avec de vrais fonds.

pub mod block_validation;
pub mod experimental;
pub mod fork_chain;
pub mod local_chain;
pub mod proof_of_work;
