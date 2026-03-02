//! SQLite storage with encryption.
//!
//! Database operations for identity, messages, contacts, and settings.

pub mod database;
pub mod identity;
pub mod messages;
pub mod contacts;
pub mod settings;

pub use database::{Database, DatabaseError};
