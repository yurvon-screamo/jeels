//! Client-side sync bookkeeping for the hybrid user repository.
//!
//! A materialized `User` with a large knowledge set is expensive in WASM
//! (megabytes of JSON to inflate, parse and re-serialize on a single
//! thread), so the repository avoids full sync cycles when nothing changed.
//! This module holds the pure decision state for that short-circuit:
//! a fingerprint of the last synchronized remote row plus a dirty flag with
//! an epoch counter that makes concurrent mutations visible.
//!
//! See ADR-045 for the full sync design and threat model.

use serde::{Deserialize, Serialize};

/// Sync bookkeeping persisted beside the local user record.
///
/// Invariants:
/// - `dirty == true` forces the next sync down the full merge path.
/// - `dirty_epoch` increments on every [`SyncMeta::mark_dirty`] call; the
///   sync orchestration captures it **after its own** `mark_dirty` and
///   passes the captured value to [`SyncMeta::record_sync`], which refuses
///   to clear the flag when any further mutation happened in the sync
///   window (lost-update protection).
/// - `last_synced_fingerprint` is computed from the **server-authoritative**
///   row bytes re-fetched after a successful push, never from the request
///   body: server-side normalization would otherwise break skip matching.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncMeta {
    pub last_synced_fingerprint: Option<String>,
    pub dirty: bool,
    pub dirty_epoch: u64,
}

impl SyncMeta {
    /// Initial state for a fresh install or a pre-sync-feature upgrade:
    /// no fingerprint and dirty, so the first sync is always a full one
    /// (fail-closed).
    pub fn unsynced() -> Self {
        Self {
            last_synced_fingerprint: None,
            dirty: true,
            dirty_epoch: 0,
        }
    }

    /// Whether the sync orchestration may skip the full merge path for a
    /// remote row with `remote_fingerprint`.
    pub fn should_skip(&self, remote_fingerprint: &str) -> bool {
        !self.dirty && self.last_synced_fingerprint.as_deref() == Some(remote_fingerprint)
    }

    /// Records a local mutation: the next sync must take the full path.
    /// Increments `dirty_epoch` so an in-flight sync notices the mutation.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.dirty_epoch += 1;
    }

    /// Records a successful full sync against the server row with
    /// `remote_fingerprint`.
    ///
    /// `observed_epoch` must be the value of `dirty_epoch` captured by the
    /// sync orchestration **after its own** `mark_dirty`: when the epoch
    /// still matches, no other mutation happened in the sync window and the
    /// dirty flag may clear; otherwise a concurrent `mark_dirty` (e.g. a
    /// card rated while the sync was pushing) keeps the flag set so the
    /// next sync re-merges and pushes the newer local state.
    pub fn record_sync(&mut self, remote_fingerprint: String, observed_epoch: u64) {
        self.last_synced_fingerprint = Some(remote_fingerprint);
        if self.dirty_epoch == observed_epoch {
            self.dirty = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synced_meta(fingerprint: &str) -> SyncMeta {
        SyncMeta {
            last_synced_fingerprint: Some(fingerprint.to_string()),
            dirty: false,
            dirty_epoch: 3,
        }
    }

    #[test]
    fn unsynced_meta_never_skips() {
        // Arrange / Act / Assert: a fresh install has no fingerprint and is
        // dirty, so the first sync is always full regardless of the remote.
        let meta = SyncMeta::unsynced();
        assert!(!meta.should_skip("anything"));
        assert!(meta.dirty);
    }

    #[test]
    fn default_meta_never_skips() {
        // A missing persisted record deserializes to default (None fp,
        // not dirty) — the orchestrator must treat it as unsynced, but the
        // type itself guarantees skip requires a fingerprint.
        assert!(!SyncMeta::default().should_skip("x"));
    }

    #[test]
    fn clean_matching_fingerprint_skips() {
        let meta = synced_meta("fp1");
        assert!(meta.should_skip("fp1"));
    }

    #[test]
    fn changed_remote_fingerprint_does_not_skip() {
        let meta = synced_meta("fp1");
        assert!(!meta.should_skip("fp2"));
    }

    #[test]
    fn dirty_meta_does_not_skip_even_with_matching_fingerprint() {
        let mut meta = synced_meta("fp1");
        meta.mark_dirty();
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn record_sync_clears_dirty_when_epoch_unchanged() {
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        let observed = meta.dirty_epoch;

        meta.record_sync("fp1".to_string(), observed);

        assert!(!meta.dirty);
        assert_eq!(meta.last_synced_fingerprint.as_deref(), Some("fp1"));
        assert!(meta.should_skip("fp1"));
    }

    #[test]
    fn record_sync_keeps_dirty_when_mutated_during_sync_window() {
        // The lost-update race: a card rated while the sync's push is in
        // flight must survive — the epoch moved past the observed value.
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        let observed = meta.dirty_epoch;
        meta.mark_dirty(); // concurrent mutation during the sync window

        meta.record_sync("fp1".to_string(), observed);

        assert!(meta.dirty, "concurrent mutation must keep the flag set");
        assert_eq!(meta.last_synced_fingerprint.as_deref(), Some("fp1"));
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn crash_before_record_sync_keeps_meta_dirty() {
        // Simulates dying after the PATCH but before the meta write: the
        // in-memory copy is dirty and the next sync is a full one (an
        // acceptable extra full sync, documented in ADR-045).
        let mut meta = SyncMeta::unsynced();
        meta.mark_dirty();
        // no record_sync call — e.g. the process died
        assert!(meta.dirty);
        assert!(!meta.should_skip("fp1"));
    }

    #[test]
    fn mark_dirty_increments_epoch_every_time() {
        let mut meta = synced_meta("fp1");
        let before = meta.dirty_epoch;
        meta.mark_dirty();
        meta.mark_dirty();
        assert_eq!(meta.dirty_epoch, before + 2);
    }
}
