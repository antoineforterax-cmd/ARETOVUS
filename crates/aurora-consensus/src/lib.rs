//! `aurora-consensus` : bloc expérimental et premières expériences de preuve de travail.
//!
//! **Ce crate n'implémente toujours pas un consensus complet.** Le helper PoW est un outil de
//! laboratoire : aucune difficulté de chaîne, récompense, règle temporelle, sélection par travail
//! cumulé, validation d'état, ni réseau pair-à-pair n'est implémenté. Les décisions restent ouvertes
//! dans `docs/DECISION_REGISTER.md`.
//!
//! Ne pas utiliser avec de vrais fonds.

pub mod block_validation;
pub mod experimental;
pub mod local_chain;
pub mod proof_of_work;
