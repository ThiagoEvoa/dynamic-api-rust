//! Shared application state: the in-memory JSON document store.
//!
//! Mirrors the Dart reference's global `dynamic json = {}`. A single shared
//! `Arc<Mutex<serde_json::Value>>` is cheap to clone (the `Arc` is shared), so
//! it satisfies axum's `State` extractor requirements (`Send + Sync + 'static`).

use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{Map, Value};

/// Process-wide store holding the current JSON document.
///
/// Constructed by [`AppState::new`], which seeds it with an empty object
/// (`{}`), matching the Dart reference.
#[derive(Clone, Default, Debug)]
pub struct AppState {
    /// The shared JSON document. Reads clone under a short read lock; writes
    /// replace the whole document under a short write lock.
    store: Arc<Mutex<Value>>,
}

impl AppState {
    /// Creates a store initialized to an empty JSON object `{}`.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Value::Object(Map::new()))),
        }
    }

    /// Returns a clone of the currently stored JSON.
    pub fn current(&self) -> Value {
        // A cloned snapshot keeps the critical section short (O(1) for a
        // short-lived guard) and avoids handing callers a guard into shared
        // mutable state.
        self.lock().clone()
    }

    /// Replaces the stored JSON with `value`, mirroring `json = request.json()`.
    pub fn replace(&self, value: Value) {
        *self.lock() = value;
    }

    /// Locks the document, recovering from a poisoned mutex so that a panic in
    /// one request cannot permanently wedge the store for all others.
    fn lock(&self) -> MutexGuard<'_, Value> {
        // SAFETY reasoning: recovering via `into_inner` is sound here because
        // the only data is an owned `Value` (no invariants a panic could leave
        // half-updated). A poisoned guard means a previous holder panicked; the
        // `Value` it left is still a complete, valid document.
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl std::ops::Deref for AppState {
    type Target = Arc<Mutex<Value>>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
