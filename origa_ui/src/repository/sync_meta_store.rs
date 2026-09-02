//! Persistence for [`SyncMeta`] — the sync short-circuit bookkeeping
//! (ADR-045).
//!
//! The meta record lives as a `sync_meta` key **inside the existing `users`
//! IndexedDB store**, next to the `user:{id}` records: the store uses
//! out-of-line keys without a key path, so no schema version bump or
//! upgrade migration is needed, and `list_users` bounds its key range to
//! the `user:` prefix (see `file_repository`) so the meta record never
//! shows up as a "corrupted user entry".

use std::future::Future;

use origa::domain::OrigaError;
use origa::use_cases::SyncMeta;

use super::file_repository::{STORE_NAME, open_database};

/// Key of the sync meta record inside the `users` object store.
pub(crate) const SYNC_META_KEY: &str = "sync_meta";

pub(crate) trait SyncMetaStore {
    /// Loads the persisted meta; a missing or corrupted record resolves to
    /// [`SyncMeta::unsynced`] — fail-closed towards a full sync.
    fn load(&self) -> impl Future<Output = Result<SyncMeta, OrigaError>>;

    fn store(&self, meta: &SyncMeta) -> impl Future<Output = Result<(), OrigaError>>;
}

/// IndexedDB-backed store (production).
#[derive(Clone, Default)]
pub(crate) struct IdbSyncMetaStore;

impl SyncMetaStore for IdbSyncMetaStore {
    async fn load(&self) -> Result<SyncMeta, OrigaError> {
        use idb::TransactionMode;

        let db = open_database().await?;
        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| {
                let reason = format!("Failed to create sync-meta read transaction: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;
        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {e:?}");
            tracing::error!("{reason}");
            OrigaError::RepositoryError { reason }
        })?;

        let value = store
            .get(wasm_bindgen::JsValue::from_str(SYNC_META_KEY))
            .map_err(|e| {
                let reason = format!("Failed to create sync-meta get request: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?
            .await
            .map_err(|e| {
                let reason = format!("Failed to read sync meta: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;

        match value {
            Some(js) => match serde_wasm_bindgen::from_value::<SyncMeta>(js) {
                Ok(meta) => Ok(meta),
                Err(e) => {
                    // Corrupted meta fails closed: a full sync is always a
                    // safe fallback, unlike trusting a half-written record.
                    tracing::warn!("Corrupted sync meta, falling back to full sync: {e:?}");
                    Ok(SyncMeta::unsynced())
                },
            },
            None => Ok(SyncMeta::unsynced()),
        }
    }

    async fn store(&self, meta: &SyncMeta) -> Result<(), OrigaError> {
        use idb::TransactionMode;

        let value = serde_wasm_bindgen::to_value(meta).map_err(|e| {
            let reason = format!("Failed to serialize sync meta: {e:?}");
            tracing::error!("{reason}");
            OrigaError::RepositoryError { reason }
        })?;

        let db = open_database().await?;
        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| {
                let reason = format!("Failed to create sync-meta write transaction: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;
        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {e:?}");
            tracing::error!("{reason}");
            OrigaError::RepositoryError { reason }
        })?;

        store
            .put(
                &value,
                Some(&wasm_bindgen::JsValue::from_str(SYNC_META_KEY)),
            )
            .map_err(|e| {
                let reason = format!("Failed to create sync-meta put request: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?
            .await
            .map_err(|e| {
                let reason = format!("Failed to write sync meta: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;

        Ok(())
    }
}

#[cfg(test)]
/// In-memory store for native orchestration tests: mirrors the IDB
/// semantics (missing → unsynced) without JavaScript.
#[derive(Clone, Default)]
pub(crate) struct InMemorySyncMetaStore {
    state: std::sync::Arc<std::sync::Mutex<Option<SyncMeta>>>,
}

#[cfg(test)]
impl SyncMetaStore for InMemorySyncMetaStore {
    async fn load(&self) -> Result<SyncMeta, OrigaError> {
        Ok(self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .unwrap_or_else(SyncMeta::unsynced))
    }

    async fn store(&self, meta: &SyncMeta) -> Result<(), OrigaError> {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(meta.clone());
        Ok(())
    }
}

#[cfg(test)]
impl InMemorySyncMetaStore {
    /// Synchronous dirty-marking for test hooks that fire inside an
    /// already-running executor (a nested `block_on` would panic).
    pub(crate) fn mark_dirty_direct(&self) {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let meta = guard.get_or_insert_with(SyncMeta::unsynced);
        meta.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_store_missing_record_resolves_to_unsynced() {
        let store = InMemorySyncMetaStore::default();
        let loaded = futures::executor::block_on(store.load()).expect("load");
        assert!(loaded.dirty);
        assert!(loaded.last_synced_fingerprint.is_none());
    }

    #[test]
    fn in_memory_store_roundtrip() {
        let store = InMemorySyncMetaStore::default();
        let meta = SyncMeta {
            last_synced_fingerprint: Some("fp".to_string()),
            dirty: false,
            dirty_epoch: 4,
        };
        futures::executor::block_on(store.store(&meta)).expect("store");
        let loaded = futures::executor::block_on(store.load()).expect("load");
        assert_eq!(loaded, meta);
    }
}
