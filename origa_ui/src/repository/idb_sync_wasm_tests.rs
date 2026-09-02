//! WASM (browser) tests for the IndexedDB pieces of the sync short-circuit
//! (ADR-045): the `sync_meta` record living beside `user:*` keys, the key
//! range that keeps it out of user listings, and the cheap
//! `has_any_user` existence probe.
//!
//! Each test resets the `origa` database first: wasm-bindgen tests share
//! one browser page (and therefore one IndexedDB origin), so hermetic
//! setups need explicit cleanup.

#![cfg(all(target_arch = "wasm32", test))]

use origa::domain::{NativeLanguage, User};
use origa::traits::UserRepository;
use origa::use_cases::SyncMeta;
use wasm_bindgen_test::*;

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::sync_meta_store::{IdbSyncMetaStore, SYNC_META_KEY, SyncMetaStore};

wasm_bindgen_test_configure!(run_in_browser);

async fn reset_database() {
    let factory = idb::Factory::new().expect("idb factory");
    let request = factory
        .delete(crate::repository::file_repository::DB_NAME)
        .expect("delete request");
    request.await.expect("database deleted");
}

#[wasm_bindgen_test]
async fn sync_meta_roundtrip_and_missing_fallback() {
    reset_database().await;
    let store = IdbSyncMetaStore;

    // Missing record resolves to unsynced (fail-closed to a full sync).
    let loaded = store.load().await.expect("load missing");
    assert!(loaded.dirty);
    assert!(loaded.last_synced_fingerprint.is_none());

    // Roundtrip.
    let meta = SyncMeta {
        last_synced_fingerprint: Some("abc123".to_string()),
        dirty: false,
        dirty_epoch: 7,
    };
    store.store(&meta).await.expect("store");
    assert_eq!(store.load().await.expect("load stored"), meta);
}

#[wasm_bindgen_test]
async fn sync_meta_key_stays_out_of_user_listings() {
    reset_database().await;

    let repo = FileSystemUserRepository::new();
    let user = User::new(
        "range-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    repo.save(&user).await.expect("user saved");

    // A sync meta record beside the user keys.
    let store = IdbSyncMetaStore;
    store
        .store(&SyncMeta {
            last_synced_fingerprint: Some("fp".to_string()),
            dirty: false,
            dirty_epoch: 1,
        })
        .await
        .expect("meta stored");

    // The user listing must see exactly the user — the meta key must not
    // surface as a "corrupted user entry" (which would also break the
    // `users.into_iter().next()` selection on some orderings).
    let current = repo.get_current_user().await.expect("get");
    assert_eq!(current.expect("user present").id(), user.id());

    // The existence probe agrees.
    use crate::repository::hybrid_repository::LocalUserPresence;
    assert!(repo.has_any_user().await.expect("has_any_user"));

    // And the meta survives alongside the user record.
    let reloaded = store.load().await.expect("meta reload");
    assert_eq!(reloaded.last_synced_fingerprint.as_deref(), Some("fp"));
}
