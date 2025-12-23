use std::sync::LazyLock;

use crate::domain::{
    JeersError,
    furiganizer::{FuriganaFormat, Furiganizer},
};

pub trait IsJapanese {
    fn is_japanese(&self) -> bool;
    fn is_hiragana(&self) -> bool;
    fn is_katakana(&self) -> bool;
    fn is_kanji(&self) -> bool;
}

pub trait IsJapaneseText {
    fn is_japanese(&self) -> bool;
    fn contains_japanese(&self) -> bool;
    fn contains_kanji(&self) -> bool;

    fn has_furigana(&self) -> Result<bool, JeersError>;
    fn as_furigana(&self) -> Result<String, JeersError>;
    fn equals_by_reading(&self, other: &Self) -> Result<bool, JeersError>;
}

impl IsJapanese for char {
    fn is_japanese(&self) -> bool {
        self.is_hiragana() || self.is_katakana() || self.is_kanji()
    }

    fn is_hiragana(&self) -> bool {
        ('\u{3040}'..='\u{309F}').contains(self)
    }

    fn is_katakana(&self) -> bool {
        ('\u{30A0}'..='\u{30FF}').contains(self)
    }

    fn is_kanji(&self) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(self)
            || ('\u{3400}'..='\u{4DBF}').contains(self)
            || ('\u{20000}'..='\u{2A6DF}').contains(self)
    }
}

static FURIGANIZER: LazyLock<Furiganizer> =
    LazyLock::new(|| Furiganizer::new(FuriganaFormat::Html).unwrap());

impl IsJapaneseText for str {
    fn is_japanese(&self) -> bool {
        self.chars().all(|c| c.is_japanese())
    }

    fn contains_japanese(&self) -> bool {
        self.chars().any(|c| c.is_japanese())
    }

    fn as_furigana(&self) -> Result<String, JeersError> {
        FURIGANIZER.furiganize(self)
    }

    fn has_furigana(&self) -> Result<bool, JeersError> {
        self.as_furigana().map(|furigana| furigana != self)
    }

    fn equals_by_reading(&self, other: &Self) -> Result<bool, JeersError> {
        let left = self.as_furigana()?;
        let right = other.as_furigana()?;
        Ok(left == right)
    }

    fn contains_kanji(&self) -> bool {
        self.chars().any(|c| c.is_kanji())
    }
}
