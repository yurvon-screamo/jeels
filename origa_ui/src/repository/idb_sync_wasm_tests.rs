//! WASM (browser) tests for the IndexedDB pieces of the sync short-circuit
//! (ADR-045): the `sync_meta` record living beside `user:*` keys, the key
//! range that keeps it out of user listings, and the cheap
//! `has_any_user` existence probe.
//!
//! The tests share one browser page (and therefore one IndexedDB origin)
//! with every other wasm test, so cleanup must be **record-level**:
//! deleting the whole database would block forever on any connection held
//! open elsewhere in the suite (IndexedDB `deleteDatabase` fires `blocked`
//! while connections are open) — exactly the hang this suite must avoid.

#![cfg(all(target_arch = "wasm32", test))]

use origa::domain::{NativeLanguage, User};
use origa::traits::UserRepository;
use origa::use_cases::SyncMeta;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::sync_meta_store::{IdbSyncMetaStore, SYNC_META_KEY, SyncMetaStore};

wasm_bindgen_test_configure!(run_in_browser);

/// Record-level cleanup: removes only the `sync_meta` key. Never touches
/// the database itself (see the module documentation for why).
async fn clear_sync_meta() {
    use idb::TransactionMode;

    use crate::repository::file_repository::{STORE_NAME, open_database};

    let db = open_database().await.expect("open database");
    let transaction = db
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .expect("read-write transaction");
    let store = transaction.object_store(STORE_NAME).expect("object store");

    store
        .delete(JsValue::from_str(SYNC_META_KEY))
        .expect("delete request")
        .await
        .expect("sync_meta deleted");
}

#[wasm_bindgen_test]
async fn sync_meta_roundtrip_and_missing_fallback() {
    clear_sync_meta().await;
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
    clear_sync_meta().await;

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

    // The user listing stays healthy with the meta record present: a user
    // is returned (the meta key must never surface as a "corrupted user
    // entry" that would break parsing) — which user depends on key order
    // across the shared suite, so presence is the assertion.
    let current = repo.get_current_user().await.expect("get");
    assert!(current.is_some(), "a user must be listed beside sync_meta");

    // The existence probe agrees (inherent method on the repository).
    assert!(repo.has_any_user().await.expect("has_any_user"));

    // And the meta survives alongside the user records.
    let reloaded = store.load().await.expect("meta reload");
    assert_eq!(reloaded.last_synced_fingerprint.as_deref(), Some("fp"));
}

/// Record-level cleanup for `user:*` keys used by the codec tests.
async fn clear_user_keys(ids: &[ulid::Ulid]) {
    use idb::TransactionMode;

    use crate::repository::file_repository::{STORE_NAME, open_database, user_key};

    let db = open_database().await.expect("open database");
    let transaction = db
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .expect("read-write transaction");
    let store = transaction.object_store(STORE_NAME).expect("object store");
    for id in ids {
        store
            .delete(JsValue::from_str(&user_key(*id)))
            .expect("delete request")
            .await
            .expect("user key deleted");
    }
}

/// The user codec (#492): records must be stored as flat JSON strings (the
/// structured-clone object graph stalled multi-MB puts), and legacy
/// object-format records written before the switch must still read back.
#[wasm_bindgen_test]
async fn user_records_store_json_string_and_read_legacy_objects() {
    use idb::TransactionMode;

    use crate::repository::file_repository::{STORE_NAME, open_database, user_key};

    let string_user = User::new(
        "codec-binary@origa.local".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let legacy_user = User::new(
        "codec-legacy@origa.local".to_string(),
        NativeLanguage::English,
        None,
    );
    clear_user_keys(&[string_user.id(), legacy_user.id()]).await;

    let repo = FileSystemUserRepository::new();

    // Current format: save → the stored value is a flat string, not an object.
    repo.save(&string_user).await.expect("string save");
    {
        let db = open_database().await.expect("open database");
        let tx = db
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .expect("read-only transaction");
        let store = tx.object_store(STORE_NAME).expect("object store");
        let value = store
            .get(JsValue::from_str(&user_key(string_user.id())))
            .expect("get request")
            .await
            .expect("stored value")
            .expect("value present");
        assert!(
            value.as_string().is_some(),
            "the stored user record must be a flat JSON string"
        );
    }

    // Legacy format: a pre-switch structured-clone object written directly.
    {
        let legacy_value = serde_wasm_bindgen::to_value(&legacy_user).expect("legacy serialize");
        let db = open_database().await.expect("open database");
        let tx = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .expect("read-write transaction");
        let store = tx.object_store(STORE_NAME).expect("object store");
        store
            .put(
                &legacy_value,
                Some(&JsValue::from_str(&user_key(legacy_user.id()))),
            )
            .expect("legacy put")
            .await
            .expect("legacy stored");
    }

    // Both records decode through the listing path (key order decides which
    // one `get_current_user` returns first, so assert presence only).
    let current = repo
        .get_current_user()
        .await
        .expect("get after both formats");
    assert!(current.is_some(), "records in either format must list");

    clear_user_keys(&[string_user.id(), legacy_user.id()]).await;
}
