//! `aretovus-core` : briques communes du **prototype expérimental** AURORA.
//!
//! Ce crate contient :
//! - [`amount`] : montants entiers (jamais de flottants) ;
//! - [`codec`] : sérialisation canonique déterministe ;
//! - [`hash`] : hachage SHA-256 avec séparation de domaines ;
//! - [`network`] : identifiants des environnements (regtest, testnet, mainnet) ;
//! - [`experimental`] : transaction jetable, qui ne préjuge d'aucune décision du registre.
//!
//! **Statut : expérimental.** Ce code n'est pas un protocole validé, n'est pas audité et ne doit
//! jamais être utilisé avec de vrais fonds. Les choix qu'il contient sont des hypothèses listées
//! dans `docs/DECISION_REGISTER.md` (annexe B).

pub mod accounts;
pub mod amount;
pub mod codec;
pub mod experimental;
pub mod hash;
pub mod merkle;
pub mod network;
pub mod signature;
pub mod state;
