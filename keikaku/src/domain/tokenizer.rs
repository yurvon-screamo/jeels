use crate::domain::JeersError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    pub orthographic_base_form: String,
    pub phonological_base_form: String,
    pub orthographic_surface_form: String,
    pub phonological_surface_form: String,
    pub part_of_speech: PartOfSpeech,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PartOfSpeech {
    Verb,              // Глагол
    Noun,              // Существительное
    IAdjective,        // И-прилагательное
    NaAdjective,       // На-прилагательное
    Adverb,            // Детерминатив
    PreNounAdjectival, // Предикатив
    Conjunction,       // Союз
    Interjection,      // Междометие
    Prefix,            // Префикс
    Suffix,            // Суффикс
    Particle,          // Частица
    AuxiliaryVerb,     // Вспомогательный глагол
    Pronoun,           // Местоимение
    Numeral,           // Числительное
    Determiner,        // Определитель
    Unspecified,       // Неизвестно
    Other,             // Другое
    Symbol,            // Символ
    Whitespace,        // Пробел
    AuxiliarySymbol,   // Вспомогательный символ
}

impl std::str::FromStr for PartOfSpeech {
    fn from_str(japanese: &str) -> Result<Self, JeersError> {
        Ok(match japanese {
            "動詞" => PartOfSpeech::Verb,
            "名詞" => PartOfSpeech::Noun,
            "形容詞" => PartOfSpeech::IAdjective,
            "形状詞" => PartOfSpeech::NaAdjective,
            "副詞" => PartOfSpeech::Adverb,
            "連体詞" => PartOfSpeech::PreNounAdjectival,
            "接続詞" => PartOfSpeech::Conjunction,
            "感動詞" => PartOfSpeech::Interjection,
            "接頭辞" => PartOfSpeech::Prefix,
            "接尾辞" => PartOfSpeech::Suffix,
            "助詞" => PartOfSpeech::Particle,
            "助動詞" => PartOfSpeech::AuxiliaryVerb,
            "代名詞" => PartOfSpeech::Pronoun,
            "数詞" => PartOfSpeech::Numeral,
            "限定詞" => PartOfSpeech::Determiner,
            "未特定" => PartOfSpeech::Unspecified,
            "その他" => PartOfSpeech::Other,
            "記号" => PartOfSpeech::Symbol,
            "空白" => PartOfSpeech::Whitespace,
            "補助記号" => PartOfSpeech::AuxiliarySymbol,
            _ => {
                return Err(JeersError::TokenizerError {
                    reason: format!("Unknown part of speech: '{japanese}'"),
                });
            }
        })
    }

    type Err = JeersError;
}

pub struct Tokenizer {
    tokenizer: lindera::tokenizer::Tokenizer,
}

impl Tokenizer {
    pub fn new() -> Result<Self, JeersError> {
        let dictionary =
            lindera::dictionary::load_dictionary("embedded://unidic").map_err(|e| {
                JeersError::TokenizerError {
                    reason: e.to_string(),
                }
            })?;

        let segmenter =
            lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);
        let tokenizer = lindera::tokenizer::Tokenizer::new(segmenter);

        Ok(Self { tokenizer })
    }

    pub fn tokenize(&self, text: &str) -> Result<Vec<TokenInfo>, JeersError> {
        let mut tokens = self
            .tokenizer
            .tokenize(text)
            .map_err(|e| JeersError::TokenizerError {
                reason: e.to_string(),
            })?;

        let debug = tokens.iter_mut().map(|t| t.as_value()).collect::<Vec<_>>();
        dbg!(debug);

        let token_infos = tokens
            .iter_mut()
            .map(|token| TokenInfo {
                orthographic_base_form: token.get("lexeme").unwrap_or_default().to_string(),
                phonological_base_form: token
                    .get("phonological_base_form")
                    .unwrap_or_default()
                    .to_string(),
                orthographic_surface_form: token
                    .get("orthographic_surface_form")
                    .unwrap_or_default()
                    .to_string(),
                phonological_surface_form: token
                    .get("phonological_surface_form")
                    .unwrap_or_default()
                    .to_string(),
                part_of_speech: token
                    .get("part_of_speech")
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(PartOfSpeech::Unspecified),
            })
            .collect();

        Ok(token_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_base_form_for_verb() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_base_form_for_noun() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "食べ物");
        assert_eq!(tokens[0].phonological_base_form, "タベモノ");
    }

    #[test]
    fn should_return_base_form_for_adjective() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "美味しい");
        assert_eq!(tokens[0].phonological_base_form, "オイシー");
    }

    #[test]
    fn should_return_base_form_for_hiragana() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベル");
    }

    #[test]
    fn should_return_surface_form_for_verb() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_return_surface_form_for_noun() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ物");
        assert_eq!(tokens[0].phonological_surface_form, "タベモノ");
    }

    #[test]
    fn should_return_surface_form_for_adjective() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "美味しい");
        assert_eq!(tokens[0].phonological_surface_form, "オイシー");
    }

    #[test]
    fn should_return_surface_form_for_hiragana() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "たべ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }
}
