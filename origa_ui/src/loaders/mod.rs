pub mod card_precache_loader;
pub mod data_loader;
pub mod dictionary;
pub mod furigana_dict_loader;
pub mod jlpt_content_loader;
pub mod kanji_bundle_store;
pub mod loading_message;
pub mod model_cache;
pub mod ocr_model_loader;
pub mod phrase_data_loader;
pub mod phrase_loader;
pub mod pitch_audio_loader;
pub mod precache_loader;
pub mod well_known_set_loader;
#[cfg(target_arch = "wasm32")]
pub mod whisper_model_loader;

pub use jlpt_content_loader::{get_jlpt_content, recalculate_user_jlpt_progress};
pub use ocr_model_loader::ModelLoader;
pub use well_known_set_loader::WellKnownSetLoaderImpl;
