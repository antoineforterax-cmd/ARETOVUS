//! `aretovus-core` : primitives communes du testnet expérimental ARETOVUS.
//!
//! Ce crate contient les types du **testnet natif** :
//! - [`accounts`] : transactions par comptes et versions d'autorisation ;
//! - [`state`] : soldes, nonces et transitions de comptes ;
//! - [`signature`] : vérification ML-DSA-65 v3 (Testnet) et profils anciens (Regtest) ;
//! - [`amount`] : montants entiers et borne de 500 millions ;
//! - [`codec`], [`hash`], [`merkle`] : formats, empreintes et engagements canoniques ;
//! - [`network`] : identifiants d'environnements ;
//! - [`experimental`] : anciens types de phase 0, **pas** le profil Testnet natif.
//!
//! **Statut : prototype non audité.** Les paramètres exécutés du testnet ne sont
//! plus de simples propositions de phase 0. Ils ne constituent pas pour autant
//! un protocole prêt à recevoir des fonds réels ; consulter les décisions
//! testnet et le code de consensus au SHA précis.

pub mod accounts;
pub mod amount;
pub mod codec;
pub mod experimental;
pub mod hash;
pub mod merkle;
pub mod network;
pub mod signature;
pub mod state;
