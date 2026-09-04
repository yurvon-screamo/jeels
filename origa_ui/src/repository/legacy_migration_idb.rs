//! Primitive IndexedDB operations used by the nil-user migration.
//!
//! Each helper wraps a single idb request and maps its error into
//! `OrigaError::RepositoryError` with a stable prefix so failures are easy to
//! attribute in logs. Callers are responsible for transaction lifecycle: every
//! function here operates on a store obtained from an already-open transaction,
//! so a sequence of reads/writes either commits together or not at all.

use idb::ObjectStore;
use origa::domain::{OrigaError, User};
use ulid::Ulid;
use wasm_bindgen::JsValue;

use super::migration_err;
use crate::repository::file_repository::user_key;

/// Read the user stored under the given id, if any. Returns `None` when the
/// key is absent or holds a null/undefined value. Corrupt rows are logged and
/// treated as missing so a single bad entry cannot abort the whole migration.
pub(super) async fn read_row(store: &ObjectStore, id: Ulid) -> Result<Option<User>, OrigaError> {
    let key = JsValue::from_str(&user_key(id));
    let value = store
        .get(key)
        .map_err(|e| migration_err("Migration row read failed", e))?
        .await
        .map_err(|e| migration_err("Migration row read await failed", e))?;

    let Some(value) = value.filter(|v| !v.is_null() && !v.is_undefined()) else {
        return Ok(None);
    };

    match crate::repository::file_repository::user_from_stored_value(&value) {
        Ok(user) => Ok(Some(user)),
        Err(reason) => {
            tracing::warn!("Skipping corrupted user entry for {id}: {reason}");
            Ok(None)
        },
    }
}

/// Returns true when a row exists under the canonical id. Uses `count` rather
/// than `get` so the store never materialises the value — existence is all the
/// migration needs to decide between reseat and merge.
pub(super) async fn row_exists(store: &ObjectStore, id: Ulid) -> Result<bool, OrigaError> {
    let key = idb::Query::Key(JsValue::from_str(&user_key(id)));
    let count = store
        .count(Some(key))
        .map_err(|e| migration_err("Migration row count failed", e))?
        .await
        .map_err(|e| migration_err("Migration row count await failed", e))?;
    Ok(count > 0)
}

/// Write a user under the given id, overwriting any existing row.
pub(super) async fn write_row(
    store: &ObjectStore,
    user: &User,
    id: Ulid,
) -> Result<(), OrigaError> {
    let value = crate::repository::file_repository::user_to_stored_value(user)
        .map_err(|e| migration_err("Migration serialize failed", e))?;
    let key = JsValue::from_str(&user_key(id));
    store
        .put(&value, Some(&key))
        .map_err(|e| migration_err("Migration put failed", e))?
        .await
        .map_err(|e| migration_err("Migration put await failed", e))?;
    Ok(())
}

/// Delete the row stored under the given id, if any.
pub(super) async fn delete_row(store: &ObjectStore, id: Ulid) -> Result<(), OrigaError> {
    let key = JsValue::from_str(&user_key(id));
    store
        .delete(key)
        .map_err(|e| migration_err("Migration delete failed", e))?
        .await
        .map_err(|e| migration_err("Migration delete await failed", e))?;
    Ok(())
}
