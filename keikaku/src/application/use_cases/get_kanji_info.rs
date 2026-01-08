use crate::domain::{
    dictionary::{KANJI_DB, KanjiInfo},
    error::KeikakuError,
};

pub struct GetKanjiInfoUseCase;

impl Default for GetKanjiInfoUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl GetKanjiInfoUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, kanji: &str) -> Result<KanjiInfo, KeikakuError> {
        Ok(KANJI_DB.get_kanji_info(kanji)?.to_owned())
    }
}
