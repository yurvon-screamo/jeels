use crate::domain::{JeersError, japanese::IsJapaneseText, tokenizer::Tokenizer};

pub struct Furiganizer {
    tokenizer: Tokenizer,
    format: FuriganaFormat,
}

pub enum FuriganaFormat {
    Html,
    Markdown,
}

impl Furiganizer {
    pub fn new(format: FuriganaFormat) -> Result<Self, JeersError> {
        Ok(Self {
            tokenizer: Tokenizer::new()?,
            format,
        })
    }
}

impl Furiganizer {
    pub fn furiganize(&self, text: &str) -> Result<String, JeersError> {
        let tokens = self.tokenizer.tokenize(text)?;
        let mut result = String::new();

        for token in tokens {
            let furigana = if token.orthographic_surface_form().contains_kanji() {
                match self.format {
                    FuriganaFormat::Html => self.format_html(
                        token.orthographic_surface_form(),
                        token.phonological_surface_form(),
                    ),
                    FuriganaFormat::Markdown => self.format_md(
                        token.orthographic_surface_form(),
                        token.phonological_surface_form(),
                    ),
                }
            } else {
                token.orthographic_surface_form().to_string()
            };

            result.push_str(&furigana);
        }

        Ok(result)
    }

    fn format_html(&self, base: &str, text: &str) -> String {
        format!("<ruby>{base}<rp>(</rp><rt>{text}</rt><rp>)</rp></ruby>")
    }

    fn format_md(&self, base: &str, text: &str) -> String {
        format!("[{base}]({text})")
    }
}
