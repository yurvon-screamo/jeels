use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
    use_cases::SyncMeta,
};

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::sync_meta_store::{IdbSyncMetaStore, SyncMetaStore};
use crate::repository::trailbase_repository::{RemoteUserSource, TrailBaseUserRepository};

#[cfg(test)]
#[path = "hybrid_sync_tests.rs"]
mod sync_tests;

#[derive(Clone)]
pub struct HybridUserRepository {
    local: FileSystemUserRepository,
    remote: TrailBaseUserRepository,
    meta: IdbSyncMetaStore,
}

impl HybridUserRepository {
    pub fn new() -> Self {
        Self {
            local: FileSystemUserRepository::new(),
            remote: TrailBaseUserRepository::new(),
            meta: IdbSyncMetaStore,
        }
    }

    pub async fn merge_current_user(&self) -> Result<(), OrigaError> {
        sync_merge(&self.local, &self.remote, &self.meta).await
    }

    /// Definitive remote-miss probe: does the server hold a user record for
    /// the signed-in session RIGHT NOW? Used by the login profile bootstrap
    /// to discriminate "genuinely first login" from "the merge returned
    /// without seeding the local store while the remote record is alive" —
    /// minting a fresh empty profile in the latter case shadows the
    /// canonical record on every device that logs in afterwards (#492).
    pub async fn has_remote_record(&self) -> Result<bool, OrigaError> {
        Ok(self.remote.find_current_raw().await?.is_some())
    }

    /// Marks the local user record as mutated: the next sync must take the
    /// full merge path. Called after every user-action write (`save`,
    /// `save_sync`) — the store write is a tiny IndexedDB record, accepted
    /// per ADR-045 (future work: co-locate it with the user write in one
    /// transaction).
    async fn mark_local_dirty(&self) {
        if let Err(e) = mark_dirty(&self.meta).await {
            tracing::warn!("Failed to persist sync dirty flag: {e:?}");
        }
    }

    /// Delete the remote user record only. Unlike `delete`, this does NOT
    /// swallow remote errors — account deletion must surface failures so the
    /// caller (AuthStore) can abort the flow instead of leaving the user in a
    /// half-deleted state. Local data cleanup is the caller's responsibility.
    ///
    /// `user_id` is accepted as `Option<Ulid>` — when `None`, the caller has no
    /// loaded domain `User`, which means the account is in an anomalous state
    /// (authenticated session but no User object). This is surfaced as an error
    /// rather than silently passing a nil ULID.
    pub async fn delete_remote(&self, user_id: Option<Ulid>) -> Result<(), OrigaError> {
        tracing::info!("delete_remote: Deleting remote user {:?}", user_id);
        let id = user_id.ok_or_else(|| OrigaError::RepositoryError {
            reason: "Cannot delete account: no user is currently loaded".to_string(),
        })?;
        // The sync meta belongs to the deleted account: reset it so a future
        // login cannot inherit a stale skip fingerprint.
        if let Err(e) = self.meta.store(&SyncMeta::unsynced()).await {
            tracing::warn!("Failed to reset sync meta after account deletion: {e:?}");
        }
        self.remote.delete(id).await
    }
}

/// Loads, dirties and persists the meta in one step. Returns the stored
/// state so the caller can capture `dirty_epoch` **after** its own
/// `mark_dirty` for the CAS check in `record_sync` (ADR-045).
async fn mark_dirty(meta_store: &impl SyncMetaStore) -> Result<SyncMeta, OrigaError> {
    let mut meta = meta_store.load().await?;
    meta.mark_dirty();
    meta_store.store(&meta).await?;
    Ok(meta)
}

/// Cheap existence check for the local user record, used by the sync
/// skip-path: the short-circuit must not fire when the local store is
/// missing or corrupted, because the full path is what re-seeds it
/// (ADR-045). Implemented via an IndexedDB key count — no user parsing.
pub(crate) trait LocalUserPresence {
    fn has_any_user(&self) -> impl Future<Output = Result<bool, OrigaError>>;
}

/// The sync orchestration core (ADR-045), generic over the repositories so
/// it runs against in-memory spies in native tests.
///
/// Steady state (nothing changed since the last successful sync) costs one
/// raw remote fetch plus a fingerprint comparison — the multi-megabyte
/// inflate/parse/serialize cycle of a large knowledge set never runs. Any
/// difference takes the full path: decode the remote row, merge into the
/// local user, push, then record the **server-authoritative** fingerprint
/// re-fetched after the push.
pub(crate) async fn sync_merge(
    local: &(impl UserRepository + LocalUserPresence),
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
) -> Result<(), OrigaError> {
    let raw = remote.find_current_raw().await?;

    let Some(remote_row) = raw else {
        // No remote record: seed the server from local (fresh install or a
        // remote wiped elsewhere).
        match local.get_current_user().await? {
            Some(local_user) => {
                tracing::info!("Creating remote user from local");
                remote.create(&local_user).await?;
                record_post_create_fingerprint(remote, meta_store).await?;
            },
            None => tracing::warn!("No user found locally or remotely"),
        }
        return Ok(());
    };

    // The skip requires a present local record: when the local store has
    // no user key, the merge below is the recovery path that re-seeds it
    // from the remote row, and skipping it would strand the user with a
    // clean fingerprint and no local data (ADR-045). The probe is a keyed
    // count — a corrupted-but-present record is NOT detected here (see the
    // ADR threat model for the residual risk).
    let meta = meta_store.load().await?;
    let local_exists = local.has_any_user().await?;
    if meta.should_skip(&remote_row.fingerprint) && local_exists {
        tracing::debug!("Sync skipped: remote unchanged since last sync");
        return Ok(());
    }

    let record_id = remote_row.record_id;
    // Captured before `into_user` consumes the row: the restore branch
    // below records the fetched fingerprint without a redundant push.
    let remote_fingerprint = remote_row.fingerprint.clone();
    let remote_user = remote_row.into_user()?;

    match local.get_current_user().await? {
        None => {
            tracing::info!("Restoring local user from remote");
            // The local content is seeded from the fetched row, so pushing
            // it back would ship megabytes of identical data. Write the
            // local record and record the fetched fingerprint directly;
            // concurrent server writes surface as a fingerprint change on
            // the next sync.
            let meta = mark_dirty(meta_store).await?;
            let observed_epoch = meta.dirty_epoch;
            local.save(&remote_user).await?;
            let mut meta = meta_store.load().await?;
            meta.record_sync(remote_fingerprint, observed_epoch);
            meta_store.store(&meta).await?;
            Ok(())
        },
        Some(mut local_user) => {
            tracing::info!("Merging remote into local user");
            local_user.merge(&remote_user);
            full_sync_cycle(local, remote, meta_store, local_user, record_id).await
        },
    }
}

/// The full path: mark dirty (crash-safety), write local, push remote, then
/// record the server-authoritative fingerprint with the epoch captured
/// after the own `mark_dirty` (concurrent mutations keep the flag set).
async fn full_sync_cycle(
    local: &impl UserRepository,
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
    user: User,
    record_id: i64,
) -> Result<(), OrigaError> {
    // Dirty BEFORE the local write: a crash between the local save and the
    // remote push must leave the flag set so the next sync re-pushes.
    let meta = mark_dirty(meta_store).await?;
    let observed_epoch = meta.dirty_epoch;

    local.save(&user).await?;
    remote.save_with_record_id(record_id, &user).await?;

    record_sync_fingerprint(remote, meta_store, observed_epoch).await
}

/// Re-fetches the raw row and records its fingerprint. The fingerprint is
/// server-authoritative on purpose: deriving it from the request body would
/// silently break skip matching whenever the server normalizes anything on
/// storage.
async fn record_sync_fingerprint(
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
    observed_epoch: u64,
) -> Result<(), OrigaError> {
    let mut meta = meta_store.load().await?;
    match remote.find_current_raw().await? {
        Some(fresh) => {
            meta.record_sync(fresh.fingerprint, observed_epoch);
            meta_store.store(&meta).await?;
        },
        None => tracing::warn!("Remote row vanished after push; sync meta left dirty"),
    }
    Ok(())
}

/// Post-create variant: identical epoch semantics, but no local write —
/// the local user already exists by construction.
async fn record_post_create_fingerprint(
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
) -> Result<(), OrigaError> {
    let meta = mark_dirty(meta_store).await?;
    record_sync_fingerprint(remote, meta_store, meta.dirty_epoch).await
}

impl UserRepository for HybridUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        self.local.get_current_user().await
    }

    // Local-only write on the hot path. Rating a card, marking it known, or
    // creating one are high-frequency actions; awaiting a remote round-trip
    // here would block the core study loop (especially on mobile). The local
    // write is authoritative for the device; cross-device propagation happens
    // through `save_sync` at explicit checkpoints (onboarding, imports, auth)
    // and through `merge_current_user` on login. The user id is already
    // canonical thanks to the session-derived ULID, so a local-only save is
    // correctly attributed to the right identity.
    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        tracing::info!("save: Starting local save for user {}", user.id());
        // Dirty BEFORE the local write — the same crash-safety invariant as
        // the full sync cycle: a crash between the write and the flag must
        // leave the flag set, otherwise the next sync silently skips and
        // the written data is never pushed (ADR-045).
        self.mark_local_dirty().await;
        self.local.save(user).await?;
        tracing::info!("save: Local save completed for user {}", user.id());
        Ok(())
    }

    // Explicit sync checkpoint: local + remote. Used by auth, onboarding, and
    // imports where a network round-trip is acceptable and the data must reach
    // the server before the user can switch devices.
    //
    // The local write runs first so the device stays usable offline even when
    // the network is down. Remote failures are then surfaced as `Err` instead
    // of being swallowed: a silent `Ok` here is what allowed the cross-device
    // split-progress bug, because the initial profile create would log a remote
    // error and return `Ok`, so the user moved on without a canonical remote
    // record and the next device's login found nothing to merge against.
    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        tracing::info!("save_sync: Starting save for user {}", user.id());
        // Dirty BEFORE the local write, and it stays set across the whole
        // remote push: the push takes seconds for a large knowledge set
        // (serialization + deflate + upload) — exactly the jetsam window
        // this ADR exists for — and a crash there must not leave a clean
        // meta that skips the next sync (ADR-045).
        self.mark_local_dirty().await;
        self.local.save(user).await?;
        tracing::info!("save_sync: Local save completed for user {}", user.id());

        if let Err(e) = self.remote.save(user).await {
            tracing::error!(
                "save_sync: Remote save failed for user {}: {:?}. Local save kept; surfacing error to caller.",
                user.id(),
                e
            );
            return Err(e);
        }

        tracing::info!("save_sync: Remote save completed for user {}", user.id());
        // The push succeeded but its fingerprint is not recorded here: the
        // next `merge_current_user` takes the full path (dirty) and records
        // the server-authoritative fingerprint — one fewer raw fetch per
        // checkpoint than recording inline (ADR-045).
        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        tracing::info!("delete: Deleting user {}", user_id);

        // Always delete local data first
        if let Err(e) = self.local.delete(user_id).await {
            tracing::error!("delete: Local delete failed: {:?}", e);
            return Err(e);
        }
        tracing::info!("delete: Local delete completed for user {}", user_id);

        // Try remote delete, but don't fail if it doesn't work
        match self.remote.delete(user_id).await {
            Ok(_) => tracing::info!("delete: Remote delete completed for user {}", user_id),
            Err(e) => {
                tracing::error!(
                    "delete: Remote delete failed for user {}: {:?}. Local data deleted.",
                    user_id,
                    e
                );
            },
        }

        Ok(())
    }
}
