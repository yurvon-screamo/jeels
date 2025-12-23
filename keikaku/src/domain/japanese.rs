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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn japanese_character_classification() {
        // Arrange
        let hiragana = 'あ';
        let katakana = 'ア';
        let kanji = '日';
        let latin = 'A';

        // Act + Assert
        assert!(hiragana.is_hiragana());
        assert!(hiragana.is_japanese());

        assert!(katakana.is_katakana());
        assert!(katakana.is_japanese());

        assert!(kanji.is_kanji());
        assert!(kanji.is_japanese());

        assert!(!latin.is_japanese());
        assert!(!latin.is_hiragana());
        assert!(!latin.is_katakana());
        assert!(!latin.is_kanji());
    }

    #[test]
    fn japanese_text_predicates() {
        // Arrange
        let pure_japanese = "こんにちは";
        let mixed = "Hello日";
        let no_japanese = "Hello";
        let with_kanji = "日本";

        // Act + Assert
        assert!(pure_japanese.is_japanese());
        assert!(pure_japanese.contains_japanese());
        assert!(!pure_japanese.contains_kanji());

        assert!(!mixed.is_japanese());
        assert!(mixed.contains_japanese());
        assert!(mixed.contains_kanji());

        assert!(!no_japanese.contains_japanese());
        assert!(!no_japanese.contains_kanji());

        assert!(with_kanji.contains_japanese());
        assert!(with_kanji.contains_kanji());
    }

    #[test]
    fn hiragana_text_has_no_furigana() {
        // Arrange
        let input = "こんにちは";

        // Act
        let has_furigana = input.has_furigana().unwrap();

        // Assert
        assert!(!has_furigana);
    }

    #[test]
    fn kanji_text_has_furigana() {
        // Arrange
        let input = "日本語";

        // Act
        let has_furigana = input.has_furigana().unwrap();

        // Assert
        assert!(has_furigana);
    }
}
