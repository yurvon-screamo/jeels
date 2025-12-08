use crate::domain::{
    dictionary::{KANJI_DB, KanjiInfo},
    error::JeersError,
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

    pub fn execute(&self, kanji: char) -> Result<KanjiInfo, JeersError> {
        Ok(KANJI_DB.get_kanji_info(&kanji)?.to_owned())
    }
}
