//! `aurora-consensus` : bloc **expérimental** et vérifications sans état.
//!
//! **Ce crate n'implémente pas encore de consensus.** Il n'y a ni preuve de travail, ni difficulté,
//! ni récompense, ni règles temporelles, ni sélection de chaîne : ces décisions (POW-01 à POW-12,
//! MON-01 à MON-05) sont ouvertes dans `docs/DECISION_REGISTER.md`.

pub mod experimental;
