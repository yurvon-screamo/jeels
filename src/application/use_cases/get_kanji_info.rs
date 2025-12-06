use crate::domain::{
    error::JeersError,
    kanji::{KANJI_DB, KanjiCard},
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

    pub fn execute(&self, kanji: char) -> Result<KanjiCard, JeersError> {
        KANJI_DB.get_kanji_card(&kanji)
    }
}
