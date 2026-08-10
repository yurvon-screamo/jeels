use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::trailbase_repository::TrailBaseUserRepository;

#[derive(Clone)]
pub struct HybridUserRepository {
    local: FileSystemUserRepository,
    remote: TrailBaseUserRepository,
}

impl HybridUserRepository {
    pub fn new() -> Self {
        Self {
            local: FileSystemUserRepository::new(),
            remote: TrailBaseUserRepository::new(),
        }
    }

    pub async fn merge_current_user(&self) -> Result<(), OrigaError> {
        let remote_result = self.remote.find_current().await?;
        let local_result = self.local.get_current_user().await?;

        match (remote_result, local_result) {
            (Some(remote_data), None) => {
                tracing::info!("Creating local user from remote");
                self.save_local_and_sync_remote(&remote_data.0).await?;
            },
            (None, Some(local_user)) => {
                tracing::info!("Creating remote user from local");
                self.remote.save(&local_user).await?;
            },
            (Some(remote_data), Some(mut local_user)) => {
                local_user.merge(&remote_data.0);
                self.save_local_and_sync_remote(&local_user).await?;
            },
            (None, None) => {
                tracing::warn!("No user found locally or remotely");
            },
        }

        Ok(())
    }

    async fn save_local_and_sync_remote(&self, user: &User) -> Result<(), OrigaError> {
        self.local.save(user).await?;

        let updated_user =
            self.local
                .get_current_user()
                .await?
                .ok_or_else(|| OrigaError::RepositoryError {
                    reason: "User not found after local save".to_string(),
                })?;
        self.remote.save(&updated_user).await?;
        Ok(())
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
        self.remote.delete(id).await
    }
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
