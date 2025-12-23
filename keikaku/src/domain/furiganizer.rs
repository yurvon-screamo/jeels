use crate::domain::{
    JeersError,
    japanese::{IsJapanese, IsJapaneseText},
    tokenizer::Tokenizer,
};

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
        let mut result = String::new();
        let mut current_segment = String::new();
        let mut is_current_japanese = false;

        for ch in text.chars() {
            let is_japanese = ch.is_japanese();

            if current_segment.is_empty() {
                is_current_japanese = is_japanese;
                current_segment.push(ch);
            } else if is_japanese == is_current_japanese {
                current_segment.push(ch);
            } else {
                if is_current_japanese {
                    result.push_str(&self.furiganize_clear_japanese(&current_segment)?);
                } else {
                    result.push_str(&current_segment);
                }
                current_segment.clear();
                current_segment.push(ch);
                is_current_japanese = is_japanese;
            }
        }

        if !current_segment.is_empty() {
            if is_current_japanese {
                result.push_str(&self.furiganize_clear_japanese(&current_segment)?);
            } else {
                result.push_str(&current_segment);
            }
        }

        Ok(result)
    }

    fn furiganize_clear_japanese(&self, text: &str) -> Result<String, JeersError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_text_keeps_non_japanese_parts_intact() {
        // Arrange
        let furiganizer = Furiganizer::new(FuriganaFormat::Html).unwrap();
        let input = "Hello 日本語 world";

        // Act
        let output = furiganizer.furiganize(input).unwrap();

        // Assert
        assert!(output.starts_with("Hello "));
        assert!(output.ends_with(" world"));
        assert!(output.contains("日本"));
        assert!(output.contains("語"));
        assert!(output.contains("<ruby>"));
        assert_ne!(output, input);
    }

    #[test]
    fn mixed_text_without_kanji_remains_unchanged() {
        // Arrange
        let furiganizer = Furiganizer::new(FuriganaFormat::Html).unwrap();
        let input = "Hello こんにちは world";

        // Act
        let output = furiganizer.furiganize(input).unwrap();

        // Assert
        assert_eq!(output, input);
    }

    #[test]
    fn non_japanese_text_is_returned_as_is() {
        // Arrange
        let furiganizer = Furiganizer::new(FuriganaFormat::Html).unwrap();
        let input = "Hello, world! 123";

        // Act
        let output = furiganizer.furiganize(input).unwrap();

        // Assert
        assert_eq!(output, input);
    }

    #[test]
    fn japanese_is_processed_only_inside_japanese_segments() {
        // Arrange
        let furiganizer = Furiganizer::new(FuriganaFormat::Html).unwrap();
        let input = "A日B本C";

        // Act
        let output = furiganizer.furiganize(input).unwrap();

        // Assert
        assert!(output.starts_with("A"));
        assert!(output.contains("B"));
        assert!(output.ends_with("C"));
        assert!(output.contains("<ruby>"));
    }
}
