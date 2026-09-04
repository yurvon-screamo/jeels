//! Native tests for the sync orchestration core (`sync_merge`, ADR-045).
//!
//! The spies mirror the contracts of the real repositories without any
//! JavaScript: `SpyLocal` implements `UserRepository`, `SpyRemote`
//! implements `RemoteUserSource` while simulating a server that stores the
//! pushed row (and may "normalize" it — e.g. add columns — to prove the
//! recorded fingerprint is server-authoritative).

use std::sync::{Arc, Mutex};

use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;
use serde_json::{Value, json};
use ulid::Ulid;

use super::sync_merge;
use crate::repository::hybrid_repository::LocalUserPresence;
use crate::repository::sync_meta_store::{InMemorySyncMetaStore, SyncMetaStore};
use crate::repository::trailbase_repository::{
    RemoteRow, RemoteUserSource, remote_row_from_value, user_to_json,
};

/// Alias kept short for the fixtures below.
type MetaStore = InMemorySyncMetaStore;

// ═══════════════════════════════════════════════════════════════════════
// Spies
// ═══════════════════════════════════════════════════════════════════════

#[derive(Clone, Default)]
struct SpyLocal {
    state: Arc<Mutex<Option<User>>>,
    saves: Arc<Mutex<Vec<Ulid>>>,
}

impl SpyLocal {
    fn with_user(user: User) -> Self {
        Self {
            state: Arc::new(Mutex::new(Some(user))),
            saves: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn save_count(&self) -> usize {
        self.saves.lock().unwrap().len()
    }
}

impl UserRepository for SpyLocal {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        Ok(self.state.lock().unwrap().clone())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        self.saves.lock().unwrap().push(user.id());
        *self.state.lock().unwrap() = Some(user.clone());
        Ok(())
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        *self.state.lock().unwrap() = None;
        Ok(())
    }
}

impl LocalUserPresence for SpyLocal {
    async fn has_any_user(&self) -> Result<bool, OrigaError> {
        Ok(self.state.lock().unwrap().is_some())
    }
}

/// Simulated TrailBase row store. `normalize_on_save` mimics server-side
/// normalization (e.g. a column the client did not send) to prove the
/// recorded fingerprint comes from the re-fetched server bytes.
struct SpyRemote {
    rows: Arc<Mutex<Vec<Value>>>,
    fetches: Arc<Mutex<usize>>,
    pushes: Arc<Mutex<Vec<i64>>>,
    creates: Arc<Mutex<Vec<Ulid>>>,
    normalize_on_save: bool,
    fail_pushes: bool,
    /// Hook executed right after a push lands — used to simulate a
    /// concurrent user action (card rated) inside the sync window.
    on_push: Option<Box<dyn Fn() + Send + Sync>>,
}

impl SpyRemote {
    fn new(rows: Vec<Value>) -> Self {
        Self {
            rows: Arc::new(Mutex::new(rows)),
            fetches: Arc::new(Mutex::new(0)),
            pushes: Arc::new(Mutex::new(Vec::new())),
            creates: Arc::new(Mutex::new(Vec::new())),
            normalize_on_save: false,
            fail_pushes: false,
            on_push: None,
        }
    }

    fn row_for_fetch(&self) -> Result<Option<RemoteRow>, OrigaError> {
        *self.fetches.lock().unwrap() += 1;
        let rows = self.rows.lock().unwrap();
        let selected = rows
            .iter()
            .filter_map(|row| {
                let id = row.get("id").and_then(Value::as_i64)?;
                Some((id, row))
            })
            .min_by_key(|(id, _)| *id)
            .map(|(_, row)| row.clone());
        match selected {
            Some(row) => remote_row_from_value(row).map(Some),
            None => Ok(None),
        }
    }

    fn store_pushed_user(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        let trailbase_id = TRAILBASE_ID
            .with(|id| id.borrow().clone())
            .unwrap_or_else(|| "00000000-0000-0000-0000-000000000001".to_string());
        let mut body = user_to_json(user, &trailbase_id)?;
        body["id"] = json!(record_id);
        if self.normalize_on_save {
            body["server_generated_field"] = json!("normalized");
        }

        let mut rows = self.rows.lock().unwrap();
        rows.retain(|row| row.get("id").and_then(Value::as_i64) != Some(record_id));
        rows.push(body);
        Ok(())
    }
}

impl RemoteUserSource for SpyRemote {
    async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        self.row_for_fetch()
    }

    async fn save_with_record_id(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        if self.fail_pushes {
            return Err(OrigaError::RepositoryError {
                reason: "simulated push failure".to_string(),
            });
        }
        self.store_pushed_user(record_id, user)?;
        self.pushes.lock().unwrap().push(record_id);
        if let Some(hook) = &self.on_push {
            hook();
        }
        Ok(())
    }

    async fn create(&self, user: &User) -> Result<i64, OrigaError> {
        let record_id = 42;
        self.store_pushed_user(record_id, user)?;
        self.creates.lock().unwrap().push(user.id());
        Ok(record_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Fixtures
// ═══════════════════════════════════════════════════════════════════════

thread_local! {
    static TRAILBASE_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn fixture_user(email: &str) -> User {
    User::new(
        email.to_string(),
        origa::domain::NativeLanguage::Russian,
        None,
    )
}

fn fixture_row(email: &str) -> Value {
    json!({
        "id": 7,
        "trailbase_id": "018f3f21-7f9f-7bbb-ade0-8d4d9e16c7e1",
        "username": "fixture",
        "email": email,
        "native_language": 0,
        "knowledge_set": "{\"study_cards\":{},\"lesson_history\":[]}",
        "updated_at": "2026-09-02T18:00:00Z"
    })
}

/// Syncs once so the meta records the current server fingerprint, then
/// returns the store for assertions.
async fn prime_synced_state(
    local: &SpyLocal,
    remote: &SpyRemote,
    meta: &MetaStore,
) -> origa::use_cases::SyncMeta {
    sync_merge(local, remote, meta).await.expect("priming sync");
    meta.load().await.expect("meta load")
}

// ═══════════════════════════════════════════════════════════════════════
// Steady state: the short-circuit
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn unchanged_state_performs_no_writes_and_single_fetch() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    let primed = futures::executor::block_on(prime_synced_state(&local, &remote, &meta));
    assert!(!primed.dirty, "priming must clear the dirty flag");

    // Act: a second sync with no changes anywhere.
    let fetches_before = *remote.fetches.lock().unwrap();
    let pushes_before = remote.pushes.lock().unwrap().len();
    let saves_before = local.save_count();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("steady-state sync");

    // Assert: one raw fetch, zero local saves, zero pushes.
    assert_eq!(*remote.fetches.lock().unwrap(), fetches_before + 1);
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_before,
        "steady state must not PATCH"
    );
    assert_eq!(
        local.save_count(),
        saves_before,
        "steady state must not write the local user"
    );
    assert!(remote.creates.lock().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Full path scenarios
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn first_sync_is_full_and_records_server_fingerprint() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "successful full sync clears dirty");
    assert!(stored.last_synced_fingerprint.is_some());

    // The recorded fingerprint must match what the server currently holds:
    // a follow-up sync with no changes skips.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches + 1,
        "second sync must only fetch (skip), not push"
    );
}

#[test]
fn dirty_local_takes_full_path() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // A user action dirties the meta.
    futures::executor::block_on(async {
        let mut m = meta.load().await.unwrap();
        m.mark_dirty();
        meta.store(&m).await.unwrap();
    });

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    assert!(
        !remote.pushes.lock().unwrap().is_empty(),
        "dirty local must be pushed"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty);
}

#[test]
fn remote_change_takes_full_path() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // Another device changes the remote row's content.
    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["username"] = json!("changed-elsewhere");
    }

    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");
    assert!(remote.pushes.lock().unwrap().len() > pushes_before);
}

#[test]
fn no_remote_row_creates_from_local_and_records_fingerprint() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("create sync");

    assert_eq!(remote.creates.lock().unwrap().len(), 1);
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(stored.last_synced_fingerprint.is_some());

    // Second sync: remote now matches the last sync → skip.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
    assert_eq!(remote.pushes.lock().unwrap().len(), 0);
}

#[test]
fn no_users_anywhere_is_a_noop() {
    let local = SpyLocal::default();
    let remote = SpyRemote::new(vec![]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("noop sync");
    assert_eq!(*remote.fetches.lock().unwrap(), 1);
    assert!(remote.creates.lock().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Correctness details from the ADR-045 threat model
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn recorded_fingerprint_is_server_authoritative_not_request_body() {
    // The server "normalizes" pushes by adding a column the client never
    // sends. If the fingerprint were derived from the request body, the
    // next sync would take the full path forever; being derived from the
    // re-fetched server bytes, the state settles into skip.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    remote.normalize_on_save = true;
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");

    let pushes = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes,
        "second sync must skip despite server-side normalization"
    );
}

#[test]
fn concurrent_mutation_during_sync_window_keeps_dirty() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    // A card rating lands while the sync's push is in flight: the hook
    // fires from inside `save_with_record_id`, i.e. inside the window
    // between epoch capture and record_sync.
    let meta_for_hook = meta.clone();
    remote.on_push = Some(Box::new(move || {
        meta_for_hook.mark_dirty_direct();
    }));

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(
        stored.dirty,
        "a mutation inside the sync window must survive record_sync (CAS)"
    );
}

#[test]
fn failed_push_keeps_dirty_for_next_sync() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    remote.fail_pushes = true;
    let meta = MetaStore::default();

    let result = futures::executor::block_on(sync_merge(&local, &remote, &meta));
    assert!(result.is_err(), "push failure must surface");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(stored.dirty, "meta stays dirty until a push succeeds");
}

#[test]
fn duplicate_server_rows_resolve_to_smallest_id() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut older = fixture_row("a@example.com");
    older["id"] = json!(9);
    older["username"] = json!("dup-older");
    let mut newer = fixture_row("a@example.com");
    newer["id"] = json!(3);
    newer["username"] = json!("dup-newer");
    let remote = SpyRemote::new(vec![older, newer]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("sync");
    assert_eq!(*remote.pushes.lock().unwrap(), vec![3]);

    // Settle: after the min-id push the recorded fingerprint matches the
    // min-id row, so a second sync skips instead of flapping between the
    // duplicate rows.
    let fetches = *remote.fetches.lock().unwrap();
    let pushes = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
    assert_eq!(remote.pushes.lock().unwrap().len(), pushes);
}

#[test]
fn missing_local_record_takes_full_path_despite_clean_fingerprint() {
    // Regression for the skip-path guard: a clean meta plus a matching
    // fingerprint must NOT skip when the local record is gone — the full
    // path is what re-seeds the local store (ADR-045).
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // The local store is wiped (device storage loss / corruption cleanup).
    let saves_before = local.save_count();
    *local.state.lock().unwrap() = None;

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("restore sync");

    assert_eq!(
        local.save_count(),
        saves_before + 1,
        "the restore path must write the local record"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "restore settles the sync state");
}

#[test]
fn restore_from_remote_does_not_push_back() {
    // Fresh device: no local record. The seeded content comes from the
    // fetched row, so the restore must not PATCH megabytes of identical
    // data back (ADR-045).
    let local = SpyLocal::default();
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("restore sync");

    assert!(remote.pushes.lock().unwrap().is_empty(), "no push-back");
    assert!(remote.creates.lock().unwrap().is_empty());
    assert_eq!(local.save_count(), 1);

    // And it settles: a second sync skips.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
    assert!(remote.pushes.lock().unwrap().is_empty());
}
