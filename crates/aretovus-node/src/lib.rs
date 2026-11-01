//! Shared ARETOVUS node networking components.
//!
//! Protocol decoding helpers are public so independent fuzz targets can exercise the same
//! parsers used by the server and synchronization client.
pub mod p2p;
