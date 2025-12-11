use std::sync::LazyLock;

use autoruby::{annotate::Annotator, format, select};
use wana_kana::ConvertJapanese;

pub trait IsJapanese {
    fn is_japanese(&self) -> bool;
    fn is_hiragana(&self) -> bool;
    fn is_katakana(&self) -> bool;
    fn is_kanji(&self) -> bool;
}

pub trait IsJapaneseText {
    fn is_japanese(&self) -> bool;
    fn contains_japanese(&self) -> bool;

    fn has_furigana(&self) -> bool;
    fn as_furigana(&self) -> String;

    fn as_hiragana(&self) -> String;

    fn equals_by_reading(&self, other: &Self) -> bool;
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

static ANNOTATOR: LazyLock<Annotator<'static>> =
    LazyLock::new(Annotator::new_with_integrated_dictionary);

impl IsJapaneseText for str {
    fn is_japanese(&self) -> bool {
        self.chars().all(|c| c.is_japanese())
    }

    fn contains_japanese(&self) -> bool {
        self.chars().any(|c| c.is_japanese())
    }

    fn as_hiragana(&self) -> String {
        self.to_hiragana()
    }

    fn as_furigana(&self) -> String {
        ANNOTATOR
            .annotate(self)
            .render(&select::heuristic::All, &format::Markdown)
    }

    fn has_furigana(&self) -> bool {
        self.as_furigana() != self
    }

    fn equals_by_reading(&self, other: &Self) -> bool {
        self.to_hiragana() == other.to_hiragana()
    }
}
