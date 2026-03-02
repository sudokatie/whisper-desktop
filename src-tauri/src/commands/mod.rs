//! Tauri commands exposed to frontend.

use ed25519_dalek::SigningKey;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::relay::RelayClient;
use crate::storage::database::Database;

pub mod identity;
pub mod messages;
pub mod contacts;
pub mod settings;
pub mod relay;

/// Shared application state managed by Tauri.
pub struct AppState {
    /// Database connection (None if not initialized)
    pub db: Arc<Mutex<Option<Database>>>,
    /// Current signing key (None if locked)
    pub signing_key: Arc<Mutex<Option<SigningKey>>>,
    /// Relay client (None if not connected)
    pub relay: Arc<Mutex<Option<RelayClient>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
            signing_key: Arc::new(Mutex::new(None)),
            relay: Arc::new(Mutex::new(None)),
        }
    }
}
