pub trait JapaneseChar {
    fn is_japanese(&self) -> bool;
    fn is_hiragana(&self) -> bool;
    fn is_katakana(&self) -> bool;
    fn is_kanji(&self) -> bool;
}

pub trait JapaneseText {
    fn is_japanese(&self) -> bool;
    fn contains_japanese(&self) -> bool;
    fn contains_kanji(&self) -> bool;
}

impl JapaneseChar for char {
    fn is_japanese(&self) -> bool {
        self.is_hiragana() || self.is_katakana() || self.is_kanji() || is_cjk_punctuation(*self)
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

fn is_cjk_punctuation(c: char) -> bool {
    ('\u{3000}'..='\u{303F}').contains(&c) // CJK Symbols and Punctuation
        || matches!(
            c,
            '\u{3005}' | '\u{309D}' | '\u{309E}' | '\u{30FD}' | '\u{30FE}' // iteration marks
            | '\u{FF01}' | '\u{FF1F}'  // fullwidth ! ?
            | '\u{FF5E}'               // fullwidth ~
            | '\u{2026}'               // ellipsis …
            | '\u{2014}' | '\u{2015}'   // em dash, horizontal bar
        )
}

impl JapaneseText for str {
    fn is_japanese(&self) -> bool {
        self.chars().all(|c| c.is_japanese())
    }

    fn contains_japanese(&self) -> bool {
        self.chars().any(|c| c.is_japanese())
    }

    fn contains_kanji(&self) -> bool {
        self.chars().any(|c| c.is_kanji())
    }
}

/// Maps every katakana codepoint in ``text`` to its hiragana counterpart by
/// subtracting the Unicode offset (``0x60``) between the two blocks.
///
/// The range ``0x30A1..=0x30F6`` excludes ``ー`` (long vowel, ``U+30FC``), the
/// katakana middle dot ``・`` (``U+30FB``) and the voiced ``ヷ..ヺ`` row
/// (``U+30F7..=U+30FA``) on purpose: those have no clean hiragana counterpart
/// and are passed through unchanged. ``ヴ`` (``U+30F4``) lands on the
/// rarely-used but valid hiragana ``ゔ`` (``U+3094``).
///
/// Shared by ``furigana_annotator`` (reading-hint comparison) and the
/// translation pipeline (hiragana-base fallback for grammar_label resolution).
pub fn katakana_to_hiragana(text: &str) -> String {
    text.chars()
        .map(|c| {
            if ('\u{30A1}'..='\u{30F6}').contains(&c) {
                char::from_u32(c as u32 - 0x60)
                    .expect("katakana→hiragana range 0x3041..0x3096 is valid unicode")
            } else {
                c
            }
        })
        .collect()
}

/// Maps every hiragana codepoint in ``text`` to its katakana counterpart by
/// adding the Unicode offset (``0x60``) between the two blocks.
///
/// Inverse of [`katakana_to_hiragana`]; same block boundaries apply.
pub fn hiragana_to_katakana(text: &str) -> String {
    text.chars()
        .map(|c| {
            if ('\u{3041}'..='\u{3096}').contains(&c) {
                char::from_u32(c as u32 + 0x60)
                    .expect("hiragana→katakana range 0x30A1..0x30F6 is valid unicode")
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case('あ', true, false, false, true)]
    #[case('い', true, false, false, true)]
    #[case('う', true, false, false, true)]
    #[case('ア', false, true, false, true)]
    #[case('イ', false, true, false, true)]
    #[case('ウ', false, true, false, true)]
    #[case('日', false, false, true, true)]
    #[case('本', false, false, true, true)]
    #[case('語', false, false, true, true)]
    #[case('a', false, false, false, false)]
    #[case('Z', false, false, false, false)]
    #[case('1', false, false, false, false)]
    fn classifies_char_by_script_type(
        #[case] input: char,
        #[case] is_hiragana: bool,
        #[case] is_katakana: bool,
        #[case] is_kanji: bool,
        #[case] is_japanese: bool,
    ) {
        assert_eq!(input.is_hiragana(), is_hiragana);
        assert_eq!(input.is_katakana(), is_katakana);
        assert_eq!(input.is_kanji(), is_kanji);
        assert_eq!(input.is_japanese(), is_japanese);
    }

    #[rstest]
    #[case('。', false, false, false, true)]
    #[case('、', false, false, false, true)]
    #[case('「', false, false, false, true)]
    #[case('」', false, false, false, true)]
    #[case('・', false, true, false, true)]
    #[case('々', false, false, false, true)]
    #[case('ー', false, true, false, true)]
    fn classifies_cjk_punctuation_by_script_type(
        #[case] input: char,
        #[case] is_hiragana: bool,
        #[case] is_katakana: bool,
        #[case] is_kanji: bool,
        #[case] is_japanese: bool,
    ) {
        assert_eq!(input.is_hiragana(), is_hiragana);
        assert_eq!(input.is_katakana(), is_katakana);
        assert_eq!(input.is_kanji(), is_kanji);
        assert_eq!(input.is_japanese(), is_japanese);
    }

    #[test]
    fn cjk_punctuation_classifies_as_japanese() {
        let chars = [
            '。', '、', '「', '」', '『', '』', '【', '】', '〜', '々', '・', 'ー',
        ];
        for ch in chars {
            assert!(
                ch.is_japanese(),
                "'{}' (U+{:04X}) should be Japanese",
                ch,
                ch as u32
            );
        }
    }

    #[test]
    fn cjk_punctuation_excludes_from_script_categories() {
        assert!(!'。'.is_hiragana());
        assert!(!'。'.is_katakana());
        assert!(!'。'.is_kanji());
        assert!(!'々'.is_kanji());
    }

    #[rstest]
    #[case("こんにちは", true, true, false)]
    #[case("コンニチハ", true, true, false)]
    #[case("日本語", true, true, true)]
    #[case("こんにちは日本語", true, true, true)]
    #[case("Hello", false, false, false)]
    #[case("", true, false, false)]
    fn detects_japanese_and_kanji_in_text(
        #[case] input: &str,
        #[case] expected_is_japanese: bool,
        #[case] expected_contains_japanese: bool,
        #[case] expected_contains_kanji: bool,
    ) {
        assert_eq!(input.is_japanese(), expected_is_japanese);
        assert_eq!(input.contains_japanese(), expected_contains_japanese);
        assert_eq!(input.contains_kanji(), expected_contains_kanji);
    }

    #[rstest]
    #[case('？', true)]
    #[case('！', true)]
    #[case('…', true)]
    #[case('～', true)]
    #[case('―', true)]
    fn classifies_fullwidth_common_symbols_as_japanese(
        #[case] input: char,
        #[case] expected: bool,
    ) {
        assert_eq!(
            input.is_japanese(),
            expected,
            "'{}' (U+{:04X}) is_japanese should be {}",
            input,
            input as u32,
            expected
        );
    }

    #[test]
    fn does_not_break_segmentation_on_fullwidth_question_mark() {
        assert!('？'.is_japanese(), "？ should be classified as Japanese");
        assert!('！'.is_japanese(), "！ should be classified as Japanese");
        assert!('…'.is_japanese(), "… should be classified as Japanese");
    }

    #[test]
    fn katakana_to_hiragana_converts_standard_range() {
        assert_eq!(katakana_to_hiragana("タベモノ"), "たべもの");
        assert_eq!(katakana_to_hiragana("ア"), "あ");
        assert_eq!(katakana_to_hiragana("ン"), "ん");
    }

    #[test]
    fn katakana_to_hiragana_preserves_non_katakana() {
        assert_eq!(katakana_to_hiragana("hello"), "hello");
        assert_eq!(katakana_to_hiragana("あいう"), "あいう");
        assert_eq!(katakana_to_hiragana("123"), "123");
    }

    #[test]
    fn katakana_to_hiragana_preserves_prolonged_sound_mark() {
        assert_eq!(katakana_to_hiragana("バー"), "ばー");
    }

    #[test]
    fn hiragana_to_katakana_converts_standard_range() {
        assert_eq!(hiragana_to_katakana("たべもの"), "タベモノ");
        assert_eq!(hiragana_to_katakana("あ"), "ア");
        assert_eq!(hiragana_to_katakana("ん"), "ン");
    }

    #[test]
    fn hiragana_to_katakana_preserves_non_hiragana() {
        assert_eq!(hiragana_to_katakana("hello"), "hello");
        assert_eq!(hiragana_to_katakana("アイウ"), "アイウ");
        assert_eq!(hiragana_to_katakana("123"), "123");
    }
}
