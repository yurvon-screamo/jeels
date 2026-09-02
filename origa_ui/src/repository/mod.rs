pub mod cache_manager;
pub mod cdn_provider;
mod dictionary_cache;
mod file_repository;
mod hybrid_repository;
#[cfg(all(test, target_arch = "wasm32"))]
mod idb_sync_wasm_tests;
mod knowledge_set_codec;
pub(crate) mod legacy_migration;
#[cfg(test)]
pub(crate) mod session;
#[cfg(not(test))]
mod session;
mod sync_meta_store;
pub mod trailbase_auth;
pub mod trailbase_client;
pub(crate) mod trailbase_id;
mod trailbase_records;
mod trailbase_repository;
pub mod trailbase_session;
mod wire_fingerprint;

pub use cdn_provider::cdn as cdn_provider;

pub use dictionary_cache::{
    get_cached_dictionary_files, get_cached_vocabulary_rkyv, save_dictionary_files_to_cache,
    save_vocabulary_to_cache_rkyv,
};
pub use hybrid_repository::HybridUserRepository;
pub use session::{
    clear_session, clear_session_async, get_session, get_session_async,
    migrate_session_to_store_if_needed, set_last_sync_time, set_pkce_verifier_async,
    set_session_async, take_pkce_verifier_async,
};
pub use trailbase_client::{AuthError, OAuthProvider, TrailBaseClient};
pub(crate) use trailbase_id::{redact as redact_id, uuid_to_ulid};
