pub(crate) mod forms_adjective;
pub(crate) mod forms_verb;
pub(crate) mod particles;
pub mod quiz_generation;

pub(crate) use particles::is_grammatical_particle;

use crate::dictionary::grammar::{FormatAction, GrammarRule};
use crate::domain::grammar::forms_adjective::{
    to_de_form, to_garu_form, to_katta_form, to_kereba_form, to_ku_form, to_kunai_form,
    to_kunakatta_form, to_kute_form, to_na_form, to_nara_form, to_nasasou_form, to_sou_form_iadj,
    to_sou_form_naadj, to_sugiru_form,
};
use crate::domain::grammar::forms_verb::{
    to_ba_form, to_causative_form, to_causative_passive_form, to_chau_form, to_imperative_form,
    to_irasshai_form, to_kudasai_form, to_main_view, to_masen_deshita_form, to_masen_form,
    to_mashita_form, to_mashou_form, to_masu_form, to_mizenkei_form, to_nai_form, to_nasai_form,
    to_nikui_form, to_o_kudasai_form, to_o_ni_narimasu_form, to_o_shimasu_form, to_passive_form,
    to_potential_form, to_sou_form_verb, to_stem_form, to_sugiru_form_verb, to_ta_form,
    to_tai_form, to_tara_form, to_te_form, to_teru_form, to_toku_form, to_volitional_form,
    to_yasui_form, to_zu_form,
};
use crate::domain::{OrigaError, PartOfSpeech, grammar::forms_adjective::adjective_remove_postfix};

use crate::domain::tokenizer::TokenInfo;
use ulid::Ulid;

pub fn apply_format_actions(
    source_word: &str,
    actions: &[FormatAction],
    part_of_speech: &PartOfSpeech,
) -> Result<String, OrigaError> {
    actions
        .iter()
        .try_fold(source_word.to_string(), |word, rule| match rule {
            FormatAction::AdjectiveRemovePostfix {} => {
                adjective_remove_postfix(&word, part_of_speech)
            },
            FormatAction::AdjectiveToKunai {} => to_kunai_form(&word, part_of_speech),
            FormatAction::AdjectiveToKatta {} => to_katta_form(&word, part_of_speech),
            FormatAction::AdjectiveToKunakatta {} => to_kunakatta_form(&word, part_of_speech),
            FormatAction::AdjectiveToKute {} => to_kute_form(&word, part_of_speech),
            FormatAction::AdjectiveToKu {} => to_ku_form(&word, part_of_speech),
            FormatAction::AdjectiveToKereba {} => to_kereba_form(&word, part_of_speech),
            FormatAction::AdjectiveToSou {} => to_sou_form_iadj(&word, part_of_speech),
            FormatAction::AdjectiveToSugiru {} => to_sugiru_form(&word, part_of_speech),
            FormatAction::AdjectiveToNa {} => to_na_form(&word, part_of_speech),
            FormatAction::AdjectiveToDe {} => to_de_form(&word, part_of_speech),
            FormatAction::AdjectiveToNara {} => to_nara_form(&word, part_of_speech),
            FormatAction::AdjectiveToSouNa {} => to_sou_form_naadj(&word, part_of_speech),
            FormatAction::AdjectiveToNasasou {} => to_nasasou_form(&word, part_of_speech),
            FormatAction::AdjectiveToGaru {} => to_garu_form(&word, part_of_speech),

            FormatAction::VerbToTeForm {} => Ok(to_te_form(&word)),
            FormatAction::VerbToMainView {} => Ok(to_main_view(&word)),
            FormatAction::VerbToMasu {} => Ok(to_masu_form(&word)),
            FormatAction::VerbToMasen {} => Ok(to_masen_form(&word)),
            FormatAction::VerbToMashita {} => Ok(to_mashita_form(&word)),
            FormatAction::VerbToMasenDeshita {} => Ok(to_masen_deshita_form(&word)),
            FormatAction::VerbToMashou {} => Ok(to_mashou_form(&word)),
            FormatAction::VerbToStem {} => Ok(to_stem_form(&word)),
            FormatAction::VerbToMizenkei {} => Ok(to_mizenkei_form(&word)),
            FormatAction::VerbToTa {} => Ok(to_ta_form(&word)),
            FormatAction::VerbToNai {} => Ok(to_nai_form(&word)),
            FormatAction::VerbToTara {} => Ok(to_tara_form(&word)),
            FormatAction::VerbToBa {} => Ok(to_ba_form(&word)),
            FormatAction::VerbToPotential {} => Ok(to_potential_form(&word)),
            FormatAction::VerbToPassive {} => Ok(to_passive_form(&word)),
            FormatAction::VerbToCausative {} => Ok(to_causative_form(&word)),
            FormatAction::VerbToCausativePassive {} => Ok(to_causative_passive_form(&word)),
            FormatAction::VerbToImperative {} => Ok(to_imperative_form(&word)),
            FormatAction::VerbToVolitional {} => Ok(to_volitional_form(&word)),
            FormatAction::VerbToSou {} => Ok(to_sou_form_verb(&word)),
            FormatAction::VerbToZu {} => Ok(to_zu_form(&word)),
            FormatAction::VerbToTai {} => Ok(to_tai_form(&word)),
            FormatAction::VerbToYasui {} => Ok(to_yasui_form(&word)),
            FormatAction::VerbToNikui {} => Ok(to_nikui_form(&word)),
            FormatAction::VerbToSugiru {} => Ok(to_sugiru_form_verb(&word)),
            FormatAction::VerbToChau {} => Ok(to_chau_form(&word)),
            FormatAction::VerbToToku {} => Ok(to_toku_form(&word)),
            FormatAction::VerbToTeru {} => Ok(to_teru_form(&word)),
            FormatAction::VerbToONinarimasu {} => Ok(to_o_ni_narimasu_form(&word)),
            FormatAction::VerbToOKudasai {} => Ok(to_o_kudasai_form(&word)),
            FormatAction::VerbToOShimasu {} => Ok(to_o_shimasu_form(&word)),
            FormatAction::VerbToNasai {} => to_nasai_form(&word),
            FormatAction::VerbToKudasai {} => to_kudasai_form(&word),
            FormatAction::VerbToIrasshai {} => to_irasshai_form(&word),

            FormatAction::AddPostfix { postfix } => Ok(word + postfix),
            FormatAction::ReplacePostfix {
                old_postfix,
                new_postfix,
            } => Ok(word.trim_end_matches(old_postfix).to_string() + new_postfix),
            FormatAction::RemovePostfix { postfix } => {
                Ok(word.trim_end_matches(postfix).to_string())
            },
        })
}

impl GrammarRule {
    pub fn format(
        &self,
        source_word: &str,
        part_of_speech: &PartOfSpeech,
    ) -> Result<String, OrigaError> {
        let format_map = self.format_map().ok_or(OrigaError::GrammarFormatError {
            reason: "Rule has no format_map".to_string(),
        })?;

        let rules = format_map
            .get(part_of_speech)
            .ok_or(OrigaError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            })?;

        apply_format_actions(source_word, rules, part_of_speech)
    }
}

pub fn detect_format_map_rules(
    text: &str,
    tokens: &[TokenInfo],
    rules: &[GrammarRule],
) -> Vec<Ulid> {
    let mut detected = std::collections::HashSet::new();

    for token in tokens {
        if !token.part_of_speech().is_vocabulary_word() {
            continue;
        }

        let base = token.orthographic_base_form();
        let pos = token.part_of_speech();

        for rule in find_format_map_matches(base, pos, text, rules) {
            detected.insert(*rule.rule_id());
        }
    }

    detected.into_iter().collect()
}

pub fn detect_keyword_rules(text: &str, rules: &[GrammarRule]) -> Vec<Ulid> {
    let mut detected = std::collections::HashSet::new();

    for rule in rules {
        let keyword_groups = rule.keywords();
        if keyword_groups.is_empty() {
            continue;
        }

        let all_groups_match = keyword_groups.iter().all(|group| {
            if group.is_empty() {
                return false;
            }
            group.iter().any(|keyword| text.contains(keyword))
        });

        if all_groups_match {
            detected.insert(*rule.rule_id());
        }
    }

    detected.into_iter().collect()
}

pub fn detect_grammar_rules_in_text(
    text: &str,
    tokens: &[TokenInfo],
    rules: &[GrammarRule],
) -> Vec<Ulid> {
    let mut result = std::collections::HashSet::new();

    result.extend(detect_format_map_rules(text, tokens, rules));
    result.extend(detect_keyword_rules(text, rules));

    result.into_iter().collect()
}

pub(crate) fn find_format_map_matches<'a>(
    base: &str,
    pos: &PartOfSpeech,
    text: &str,
    rules: &'a [GrammarRule],
) -> Vec<&'a GrammarRule> {
    rules
        .iter()
        .filter(|rule| rule.has_format_map())
        .filter(|rule| formatted_rule_matches_text(rule, base, pos, text))
        .collect()
}

fn formatted_rule_matches_text(
    rule: &GrammarRule,
    base: &str,
    pos: &PartOfSpeech,
    text: &str,
) -> bool {
    let Ok(formatted) = rule.format(base, pos) else {
        return false;
    };
    if formatted == base {
        return false;
    }
    if text.contains(&formatted) {
        return true;
    }
    formatted_conjugation_variants(&formatted)
        .iter()
        .any(|variant| text.contains(variant))
}

fn formatted_conjugation_variants(formatted: &str) -> Vec<String> {
    let mut variants = Vec::new();
    if formatted.ends_with('る') {
        collect_verb_variants(formatted, &mut variants);
    } else if matches_i_adjective_aux_suffix(formatted) {
        collect_i_adjective_variants(formatted, &mut variants);
    }
    variants
}

fn matches_i_adjective_aux_suffix(formatted: &str) -> bool {
    const I_ADJECTIVE_AUX_SUFFIXES: &[&str] = &["たい", "やすい", "にくい"];
    I_ADJECTIVE_AUX_SUFFIXES
        .iter()
        .any(|suffix| formatted.ends_with(suffix))
}

fn collect_verb_variants(formatted: &str, variants: &mut Vec<String>) {
    variants.push(to_te_form(formatted));
    variants.push(to_ta_form(formatted));
    variants.push(to_nai_form(formatted));
    variants.push(to_masu_form(formatted));
    if let Some(stem) = formatted.strip_suffix('る') {
        // Min length guards against false-positive substring matches on
        // single-char stems (e.g. 見る → 見 matches almost any text).
        if stem.chars().count() >= 2 {
            variants.push(stem.to_string());
        }
    }
}

fn collect_i_adjective_variants(formatted: &str, variants: &mut Vec<String>) {
    if let Ok(form) = to_kute_form(formatted, &PartOfSpeech::IAdjective) {
        variants.push(form);
    }
    if let Ok(form) = to_katta_form(formatted, &PartOfSpeech::IAdjective) {
        variants.push(form);
    }
    if let Ok(form) = to_kunai_form(formatted, &PartOfSpeech::IAdjective) {
        variants.push(form);
    }
    if let Ok(form) = to_ku_form(formatted, &PartOfSpeech::IAdjective) {
        variants.push(form);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{FormatAction, GrammarRule, GrammarRuleContent, Nuances};
    use crate::domain::{JapaneseLevel, NativeLanguage, PartOfSpeech};
    use std::collections::HashMap;

    fn create_rule_with_format_map(
        format_map: HashMap<PartOfSpeech, Vec<FormatAction>>,
    ) -> GrammarRule {
        GrammarRule::new(
            Ulid::new(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Test Rule".to_string(),
                    "Test description".to_string(),
                    "Explanation".to_string(),
                    "How to form".to_string(),
                    "Examples".to_string(),
                    Nuances::default(),
                    "Pro tip".to_string(),
                    Vec::new(),
                ),
            )]),
            Some(format_map),
        )
    }

    mod format_errors {
        use super::*;

        #[test]
        fn returns_error_when_format_map_is_none() {
            let rule = GrammarRule::new(Ulid::new(), JapaneseLevel::N5, HashMap::new(), None);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                OrigaError::GrammarFormatError { .. }
            ));
        }

        #[test]
        fn returns_error_when_part_of_speech_not_supported() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                OrigaError::GrammarFormatError { .. }
            ));
        }
    }

    mod i_adjective_forms {
        use super::*;

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveRemovePostfix {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなる");
        }

        #[test]
        fn converts_to_kunai() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKunai {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くない");
        }

        #[test]
        fn converts_to_katta() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKatta {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高かった");
        }

        #[test]
        fn converts_to_kunakatta() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKunakatta {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなかった");
        }

        #[test]
        fn converts_to_kute() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKute {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くて");
        }

        #[test]
        fn converts_to_ku() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKu {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高く");
        }

        #[test]
        fn converts_to_kereba() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKereba {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高ければ");
        }

        #[test]
        fn converts_to_sou() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToSou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高そう");
        }

        #[test]
        fn converts_to_sugiru() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToSugiru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高すぎる");
        }

        #[test]
        fn converts_to_nasasou() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToNasasou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高なさそう");
        }

        #[test]
        fn converts_to_garu() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToGaru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高がる");
        }
    }

    mod na_adjective_forms {
        use super::*;

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveRemovePostfix {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かになる");
        }

        #[test]
        fn converts_to_na() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNa {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かな");
        }

        #[test]
        fn converts_to_de() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToDe {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かで");
        }

        #[test]
        fn converts_to_nara() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNara {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かなら");
        }

        #[test]
        fn converts_to_sou_na() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToSouNa {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かそう");
        }

        #[test]
        fn converts_to_nasasou() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToNasasou {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かじゃなさそう");
        }

        #[test]
        fn converts_to_garu() {
            let format_map = HashMap::from([(
                PartOfSpeech::NaAdjective,
                vec![FormatAction::AdjectiveToGaru {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("静かな", &PartOfSpeech::NaAdjective);

            assert_eq!(result.unwrap(), "静かがる");
        }
    }

    mod verb_forms {
        use super::*;

        #[test]
        fn converts_to_te_form() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行って");
        }

        #[test]
        fn converts_to_main_view() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMainView {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行き");
        }

        #[test]
        fn converts_to_masu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きます");
        }

        #[test]
        fn converts_to_masen() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasen {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きません");
        }

        #[test]
        fn converts_to_mashita() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMashita {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きました");
        }

        #[test]
        fn converts_to_masen_deshita() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToMasenDeshita {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きませんでした");
        }

        #[test]
        fn converts_to_mashou() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMashou {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きましょう");
        }

        #[test]
        fn converts_to_stem() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToStem {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行き");
        }

        #[test]
        fn converts_to_ta() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTa {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行った");
        }

        #[test]
        fn converts_to_nai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かない");
        }

        #[test]
        fn converts_to_tara() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTara {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ったら");
        }

        #[test]
        fn converts_to_ba() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToBa {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行けば");
        }

        #[test]
        fn converts_to_potential() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPotential {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ける");
        }

        #[test]
        fn converts_to_passive() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPassive {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かれる");
        }

        #[test]
        fn converts_to_causative() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToCausative {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かせる");
        }

        #[test]
        fn converts_to_causative_passive() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToCausativePassive {}],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かされる");
        }

        #[test]
        fn converts_to_imperative() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToImperative {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行け");
        }

        #[test]
        fn converts_to_volitional() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToVolitional {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行こう");
        }

        #[test]
        fn converts_to_sou() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToSou {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きそう");
        }

        #[test]
        fn converts_to_zu() {
            let format_map = HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToZu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行かず");
        }

        #[test]
        fn converts_to_mizenkei() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMizenkei {}])]);
            let rule = create_rule_with_format_map(format_map);

            assert_eq!(rule.format("行く", &PartOfSpeech::Verb).unwrap(), "行か");
            assert_eq!(rule.format("食べる", &PartOfSpeech::Verb).unwrap(), "食べ");
            assert_eq!(rule.format("する", &PartOfSpeech::Verb).unwrap(), "せ");
            assert_eq!(rule.format("くる", &PartOfSpeech::Verb).unwrap(), "こ");
        }

        #[test]
        fn mizenkei_zaru_chain_correct_for_suru() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![
                    FormatAction::VerbToMizenkei {},
                    FormatAction::AddPostfix {
                        postfix: "ざる".to_string(),
                    },
                ],
            )]);
            let rule = create_rule_with_format_map(format_map);

            assert_eq!(
                rule.format("書く", &PartOfSpeech::Verb).unwrap(),
                "書かざる"
            );
            assert_eq!(
                rule.format("食べる", &PartOfSpeech::Verb).unwrap(),
                "食べざる"
            );
            assert_eq!(
                rule.format("する", &PartOfSpeech::Verb).unwrap(),
                "せざる",
                "する must give せざる, not しざる"
            );
            assert_eq!(rule.format("くる", &PartOfSpeech::Verb).unwrap(), "こざる");
        }

        #[test]
        fn converts_to_tai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きたい");
        }

        #[test]
        fn converts_to_yasui() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToYasui {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きやすい");
        }

        #[test]
        fn converts_to_nikui() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNikui {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きにくい");
        }

        #[test]
        fn converts_to_sugiru() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToSugiru {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きすぎる");
        }

        #[test]
        fn converts_to_chau() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToChau {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行っちゃう");
        }

        #[test]
        fn converts_to_toku() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToToku {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行っとく");
        }

        #[test]
        fn converts_to_teru() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeru {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行ってる");
        }

        #[test]
        fn converts_to_o_ni_narimasu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToONinarimasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きになる");
        }

        #[test]
        fn converts_to_o_kudasai() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToOKudasai {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きください");
        }

        #[test]
        fn converts_to_o_shimasu() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToOShimasu {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "お行きする");
        }

        #[test]
        fn irregular_verb_suru_conversions() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("する", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "して");
        }

        #[test]
        fn irregular_verb_kuru_conversions() {
            let format_map =
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("くる", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "きて");
        }
    }

    mod postfix_operations {
        use super::*;

        #[test]
        fn adds_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::AddPostfix {
                    postfix: "たい".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行き", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行きたい");
        }

        #[test]
        fn replaces_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::ReplacePostfix {
                    old_postfix: "く".to_string(),
                    new_postfix: "いて".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行いて");
        }

        #[test]
        fn removes_postfix() {
            let format_map = HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::RemovePostfix {
                    postfix: "く".to_string(),
                }],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("行く", &PartOfSpeech::Verb);

            assert_eq!(result.unwrap(), "行");
        }
    }

    mod chained_operations {
        use super::*;

        #[test]
        fn chains_multiple_format_actions() {
            let format_map = HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![
                    FormatAction::AdjectiveRemovePostfix {},
                    FormatAction::AddPostfix {
                        postfix: "です".to_string(),
                    },
                ],
            )]);
            let rule = create_rule_with_format_map(format_map);

            let result = rule.format("高い", &PartOfSpeech::IAdjective);

            assert_eq!(result.unwrap(), "高くなるです");
        }
    }

    fn make_verb_token(base: &str) -> TokenInfo {
        TokenInfo::new_test(base, PartOfSpeech::Verb)
    }

    fn make_iadj_token(base: &str) -> TokenInfo {
        TokenInfo::new_test(base, PartOfSpeech::IAdjective)
    }

    fn make_naadj_token(base: &str) -> TokenInfo {
        TokenInfo::new_test(base, PartOfSpeech::NaAdjective)
    }

    fn make_particle_token(base: &str) -> TokenInfo {
        TokenInfo::new_test(base, PartOfSpeech::Particle)
    }

    fn make_rule(id: &str, format_map: HashMap<PartOfSpeech, Vec<FormatAction>>) -> GrammarRule {
        GrammarRule::new(
            Ulid::from_string(id).unwrap(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    Nuances::default(),
                    "Test".to_string(),
                    Vec::new(),
                ),
            )]),
            Some(format_map),
        )
    }

    fn make_keyword_rule(id: &str, keywords: Vec<Vec<String>>) -> GrammarRule {
        GrammarRule::new_with_keywords(
            Ulid::from_string(id).unwrap(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    "Test".to_string(),
                    Nuances::default(),
                    "Test".to_string(),
                    Vec::new(),
                ),
            )]),
            None,
            keywords,
        )
    }

    mod detect_format_map_rules {
        use super::*;

        fn all_test_rules() -> Vec<GrammarRule> {
            vec![
                make_rule(
                    "01H00000000000000000000001",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]),
                ),
                make_rule(
                    "01H00000000000000000000002",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTa {}])]),
                ),
                make_rule(
                    "01H00000000000000000000003",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasu {}])]),
                ),
                make_rule(
                    "01H00000000000000000000004",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNai {}])]),
                ),
                make_rule(
                    "01H00000000000000000000005",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTeForm {},
                            FormatAction::AddPostfix {
                                postfix: "ください".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000006",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTeForm {},
                            FormatAction::AddPostfix {
                                postfix: "います".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000007",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToStem {},
                            FormatAction::AddPostfix {
                                postfix: "ながら".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000008",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToNai {},
                            FormatAction::AddPostfix {
                                postfix: "ければなりません".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000009",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTai {},
                            FormatAction::AddPostfix {
                                postfix: "です".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000010",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AddPostfix {
                            postfix: "です".to_string(),
                        }],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000011",
                    HashMap::from([(
                        PartOfSpeech::NaAdjective,
                        vec![FormatAction::AddPostfix {
                            postfix: "です".to_string(),
                        }],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000012",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AdjectiveToSugiru {}],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000013",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPotential {}])]),
                ),
                make_rule(
                    "01H00000000000000000000014",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AdjectiveToKute {}],
                    )]),
                ),
            ]
        }

        fn te_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000001").unwrap()
        }
        fn ta_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000002").unwrap()
        }
        fn masu_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000003").unwrap()
        }
        fn nai_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000004").unwrap()
        }
        fn tekudasai_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000005").unwrap()
        }
        fn teimasu_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000006").unwrap()
        }
        fn nagara_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000007").unwrap()
        }
        fn nakereba_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000008").unwrap()
        }
        fn taidesu_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000009").unwrap()
        }
        fn iadj_desu_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000010").unwrap()
        }
        fn naadj_desu_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000011").unwrap()
        }
        fn iadj_sugiru_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000012").unwrap()
        }
        fn potential_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000013").unwrap()
        }
        fn iadj_kute_id() -> Ulid {
            Ulid::from_string("01H00000000000000000000014").unwrap()
        }

        #[test]
        fn f01_detects_te_form_ichidan() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べている人", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form, got: {:?}",
                result
            );
        }

        #[test]
        fn f02_detects_ta_form_ichidan() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べた", &tokens, &rules);
            assert!(
                result.contains(&ta_id()),
                "Should detect ta-form, got: {:?}",
                result
            );
        }

        #[test]
        fn f03_detects_masu_form() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べます", &tokens, &rules);
            assert!(
                result.contains(&masu_id()),
                "Should detect masu, got: {:?}",
                result
            );
        }

        #[test]
        fn f04_detects_nai_form() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べない", &tokens, &rules);
            assert!(
                result.contains(&nai_id()),
                "Should detect nai, got: {:?}",
                result
            );
        }

        #[test]
        fn f05_detects_te_form_godan_ku() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("行く")];
            let result = detect_format_map_rules("行って", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form for 行く, got: {:?}",
                result
            );
        }

        #[test]
        fn f06_detects_te_form_godan_su() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("話す")];
            let result = detect_format_map_rules("話して", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form for 話す, got: {:?}",
                result
            );
        }

        #[test]
        fn f07_detects_te_form_godan_gu() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("泳ぐ")];
            let result = detect_format_map_rules("泳いで", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form for 泳ぐ, got: {:?}",
                result
            );
        }

        #[test]
        fn f08_detects_ta_form_godan_ku() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("書く")];
            let result = detect_format_map_rules("書いた", &tokens, &rules);
            assert!(
                result.contains(&ta_id()),
                "Should detect ta-form for 書く, got: {:?}",
                result
            );
        }

        #[test]
        fn f09_detects_te_form_and_tekudasai() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べてください", &tokens, &rules);
            assert!(result.contains(&te_id()), "Should detect te-form");
            assert!(
                result.contains(&tekudasai_id()),
                "Should detect てください, got: {:?}",
                result
            );
        }

        #[test]
        fn f10_detects_te_form_and_teimasu() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べています", &tokens, &rules);
            assert!(result.contains(&te_id()), "Should detect te-form");
            assert!(
                result.contains(&teimasu_id()),
                "Should detect ています, got: {:?}",
                result
            );
        }

        #[test]
        fn f11_detects_nagara() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べながら", &tokens, &rules);
            assert!(
                result.contains(&nagara_id()),
                "Should detect ながら, got: {:?}",
                result
            );
        }

        #[test]
        fn f12_nakereba_wrong_chain_reveal_not_detected() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("行く")];
            let result = detect_format_map_rules("行かなければなりません", &tokens, &rules);
            assert!(
                !result.contains(&nakereba_id()),
                "REGRESSION: なければなりません should NOT be detected due to wrong format_map chain, got: {:?}",
                result
            );
        }

        // F12_FIX: Verify that ReplacePostfix fixes the nai-chain issue.
        // Rule uses: [VerbToNai, ReplacePostfix { old: "ない", new: "なければなりません" }]
        #[test]
        fn f12_fix_nakereba_now_detected_with_replace_postfix() {
            let nakereba_fixed = make_rule(
                "01H00000000000000000000015",
                HashMap::from([(
                    PartOfSpeech::Verb,
                    vec![
                        FormatAction::VerbToNai {},
                        FormatAction::ReplacePostfix {
                            old_postfix: "ない".to_string(),
                            new_postfix: "なければなりません".to_string(),
                        },
                    ],
                )]),
            );
            let mut rules = all_test_rules();
            rules.push(nakereba_fixed);

            let nakereba_fixed_id = Ulid::from_string("01H00000000000000000000015").unwrap();
            let tokens = vec![make_verb_token("行く")];
            let result = detect_format_map_rules("行かなければなりません", &tokens, &rules);
            assert!(
                result.contains(&nakereba_fixed_id),
                "ReplacePostfix should detect なければなりません, got: {:?}",
                result
            );
        }

        #[test]
        fn f13_detects_tai_desu() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("食べたいです", &tokens, &rules);
            assert!(
                result.contains(&taidesu_id()),
                "Should detect たいです, got: {:?}",
                result
            );
        }

        #[test]
        fn f14_detects_iadj_desu() {
            let rules = all_test_rules();
            let tokens = vec![make_iadj_token("高い")];
            let result = detect_format_map_rules("高いです", &tokens, &rules);
            assert!(
                result.contains(&iadj_desu_id()),
                "Should detect IAdj+です, got: {:?}",
                result
            );
        }

        #[test]
        fn f15_detects_naadj_desu() {
            let rules = all_test_rules();
            let tokens = vec![make_naadj_token("静か")];
            let result = detect_format_map_rules("静かです", &tokens, &rules);
            assert!(
                result.contains(&naadj_desu_id()),
                "Should detect NaAdj+です, got: {:?}",
                result
            );
        }

        #[test]
        fn f16_detects_iadj_sugiru() {
            let rules = all_test_rules();
            let tokens = vec![make_iadj_token("高い")];
            let result = detect_format_map_rules("高すぎる", &tokens, &rules);
            assert!(
                result.contains(&iadj_sugiru_id()),
                "Should detect IAdj+すぎる, got: {:?}",
                result
            );
        }

        #[test]
        fn f17_no_kunai_rule_returns_empty() {
            let rules = all_test_rules();
            let tokens = vec![make_iadj_token("高い")];
            let result = detect_format_map_rules("高くない", &tokens, &rules);
            assert!(
                result.is_empty(),
                "No AdjectiveToKunai rule exists, should return empty, got: {:?}",
                result
            );
        }

        #[test]
        fn f18_detects_te_form_suru() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("する")];
            let result = detect_format_map_rules("している", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form for する, got: {:?}",
                result
            );
        }

        #[test]
        fn f19_detects_ta_form_kuru() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("くる")];
            let result = detect_format_map_rules("きた", &tokens, &rules);
            assert!(
                result.contains(&ta_id()),
                "Should detect ta-form for くる, got: {:?}",
                result
            );
        }

        #[test]
        fn f20_detects_potential_suru() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("する")];
            let result = detect_format_map_rules("できる", &tokens, &rules);
            assert!(
                result.contains(&potential_id()),
                "Should detect potential for する, got: {:?}",
                result
            );
        }

        #[test]
        fn f21_base_form_only_returns_empty() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる")];
            let result = detect_format_map_rules("猫は魚を食べる", &tokens, &rules);
            assert!(
                result.is_empty(),
                "Base form should not match any rule, got: {:?}",
                result
            );
        }

        #[test]
        fn f22_iadj_base_without_postfix() {
            let rules = all_test_rules();
            let tokens = vec![make_iadj_token("高い")];
            let result = detect_format_map_rules("高い", &tokens, &rules);
            assert!(
                result.is_empty(),
                "高い without postfix should not match, got: {:?}",
                result
            );
        }

        #[test]
        fn f23_naadj_da_not_desu() {
            let rules = all_test_rules();
            let tokens = vec![make_naadj_token("静か")];
            let result = detect_format_map_rules("静かだ", &tokens, &rules);
            assert!(
                result.is_empty(),
                "静かだ should not match 静かです rule, got: {:?}",
                result
            );
        }

        #[test]
        fn f24_base_form_verb_nomu() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("飲む")];
            let result = detect_format_map_rules("水を飲む", &tokens, &rules);
            assert!(
                result.is_empty(),
                "Base form 飲む should not match any rule, got: {:?}",
                result
            );
        }

        #[test]
        fn f25_multiple_tokens_multiple_rules() {
            let rules = all_test_rules();
            let tokens = vec![make_verb_token("食べる"), make_verb_token("飲む")];
            let result = detect_format_map_rules("食べて飲んだ", &tokens, &rules);
            assert!(
                result.contains(&te_id()),
                "Should detect te-form from 食べる"
            );
            assert!(
                result.contains(&ta_id()),
                "Should detect ta-form from 飲む, got: {:?}",
                result
            );
        }

        #[test]
        fn f26_multiple_iadj_rules() {
            let rules = all_test_rules();
            let tokens = vec![make_iadj_token("高い"), make_iadj_token("美味しい")];
            let result = detect_format_map_rules("高くて美味しいです", &tokens, &rules);
            assert!(
                result.contains(&iadj_kute_id()),
                "Should detect くて from 高い"
            );
            assert!(
                result.contains(&iadj_desu_id()),
                "Should detect いです from 美味しい, got: {:?}",
                result
            );
        }
    }

    mod find_format_map_matches {
        use super::*;

        #[test]
        fn returns_empty_for_no_format_map_rules() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000030",
                vec![vec!["て".to_string()]],
            )];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べて",
                &rules,
            );
            assert!(result.is_empty(), "Keyword-only rules should not match");
        }

        #[test]
        fn returns_empty_when_formatted_not_found_in_text() {
            let rules = vec![create_rule_with_format_map(HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToTeForm {}],
            )]))];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "魚を食べる",
                &rules,
            );
            // "食べて" is not contained in "魚を食べる", so no match
            assert!(
                result.is_empty(),
                "Formatted form not present in text should produce no matches"
            );
        }

        #[test]
        fn returns_empty_for_wrong_pos() {
            let rules = vec![create_rule_with_format_map(HashMap::from([(
                PartOfSpeech::IAdjective,
                vec![FormatAction::AdjectiveToKu {}],
            )]))];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べて",
                &rules,
            );
            assert!(
                result.is_empty(),
                "Adjective rule should not match Verb POS"
            );
        }

        #[test]
        fn returns_match_for_correct_form() {
            let rules = vec![create_rule_with_format_map(HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToTeForm {}],
            )]))];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べて",
                &rules,
            );
            assert_eq!(result.len(), 1, "Should match te-form rule");
        }

        #[test]
        fn returns_multiple_matches_for_composite_text() {
            let rules = vec![
                create_rule_with_format_map(HashMap::from([(
                    PartOfSpeech::Verb,
                    vec![FormatAction::VerbToTeForm {}],
                )])),
                create_rule_with_format_map(HashMap::from([(
                    PartOfSpeech::Verb,
                    vec![
                        FormatAction::VerbToTeForm {},
                        FormatAction::AddPostfix {
                            postfix: "ください".to_string(),
                        },
                    ],
                )])),
            ];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べてください",
                &rules,
            );
            assert!(
                result.len() >= 2,
                "Should match both bare te-form and te+kudasai"
            );
        }

        fn create_sugiru_rule() -> GrammarRule {
            create_rule_with_format_map(HashMap::from([
                (
                    PartOfSpeech::IAdjective,
                    vec![FormatAction::AdjectiveToSugiru {}],
                ),
                (PartOfSpeech::Verb, vec![FormatAction::VerbToSugiru {}]),
            ]))
        }

        fn create_tai_rule() -> GrammarRule {
            create_rule_with_format_map(HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToTai {}],
            )]))
        }

        #[test]
        fn detects_sugiru_te_form_via_conjugation_chain() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "強すぎて寒い",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect すぎる via te-form conjugation (強すぎて)"
            );
        }

        #[test]
        fn detects_sugiru_ta_form_via_conjugation_chain() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "強すぎた",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect すぎる via ta-form conjugation (強すぎた)"
            );
        }

        #[test]
        fn detects_sugiru_stem_form_via_conjugation_chain() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "強すぎ見える",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect すぎる via stem form (強すぎ)"
            );
        }

        #[test]
        fn detects_sugiru_masu_form_via_conjugation_chain() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "強すぎますね",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect すぎる via masu-form (強すぎます)"
            );
        }

        #[test]
        fn detects_tai_te_form_via_conjugation_chain() {
            let rules = vec![create_tai_rule()];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べたくて",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect たい via te-form conjugation (食べたくて)"
            );
        }

        #[test]
        fn detects_tai_katta_form_via_conjugation_chain() {
            let rules = vec![create_tai_rule()];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べたかった",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect たい via katta-form (食べたかった)"
            );
        }

        #[test]
        fn detects_tai_kunai_form_via_conjugation_chain() {
            let rules = vec![create_tai_rule()];
            let result = super::super::find_format_map_matches(
                "食べる",
                &PartOfSpeech::Verb,
                "食べたくない",
                &rules,
            );
            assert!(
                !result.is_empty(),
                "Should detect たい via kunai-form (食べたくない)"
            );
        }

        #[test]
        fn does_not_match_unrelated_text_via_conjugation_chain() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "猫は魚を食べる",
                &rules,
            );
            assert!(
                result.is_empty(),
                "No conjugated form of 強すぎる should appear in unrelated text"
            );
        }

        #[test]
        fn still_detects_direct_match_for_sugiru_base() {
            let rules = vec![create_sugiru_rule()];
            let result = super::super::find_format_map_matches(
                "強い",
                &PartOfSpeech::IAdjective,
                "強すぎる問題",
                &rules,
            );
            assert!(!result.is_empty(), "Direct 強すぎる match must still work");
        }
    }

    mod conjugation_variants {
        #[test]
        fn generates_verb_variants_for_sugiru_form() {
            let variants = super::formatted_conjugation_variants("強すぎる");
            assert!(variants.contains(&"強すぎて".to_string()));
            assert!(variants.contains(&"強すぎた".to_string()));
            assert!(variants.contains(&"強すぎない".to_string()));
            assert!(variants.contains(&"強すぎます".to_string()));
            assert!(variants.contains(&"強すぎ".to_string()));
        }

        #[test]
        fn generates_i_adjective_variants_for_tai_form() {
            let variants = super::formatted_conjugation_variants("食べたい");
            assert!(variants.contains(&"食べたくて".to_string()));
            assert!(variants.contains(&"食べたかった".to_string()));
            assert!(variants.contains(&"食べたくない".to_string()));
            assert!(variants.contains(&"食べたく".to_string()));
        }

        #[test]
        fn returns_empty_for_non_conjugatable_form() {
            let variants = super::formatted_conjugation_variants("食べてください");
            assert!(
                variants.is_empty(),
                "Forms ending in neither る nor い should produce no variants"
            );
        }
    }

    mod detect_keyword_rules {
        use super::*;

        #[test]
        fn k01_detects_nagara_keyword() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000020",
                vec![vec!["ながら".to_string()]],
            )];
            let result = detect_keyword_rules("食べながら勉強する", &rules);
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn k02_no_keyword_match() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000020",
                vec![vec!["ながら".to_string()]],
            )];
            let result = detect_keyword_rules("猫は魚だ", &rules);
            assert!(result.is_empty());
        }

        #[test]
        fn k03_detects_teiru_keyword() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000021",
                vec![vec!["ている".to_string(), "てる".to_string()]],
            )];
            let result = detect_keyword_rules("食べている", &rules);
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn k04_detects_teru_variation() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000021",
                vec![vec!["ている".to_string(), "てる".to_string()]],
            )];
            let result = detect_keyword_rules("食べてる", &rules);
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn k05_and_logic_all_groups_required() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000022",
                vec![vec!["しか".to_string()], vec!["ない".to_string()]],
            )];
            let result = detect_keyword_rules("残念だが普通の焼酎しかない", &rules);
            assert_eq!(result.len(), 1, "Both しか and ない present → match");

            let result2 = detect_keyword_rules("普通の焼酎しか", &rules);
            assert!(result2.is_empty(), "Only しか, missing ない → no match");
        }

        #[test]
        fn k06_empty_keywords_skipped() {
            let rules = vec![make_keyword_rule("01H00000000000000000000023", vec![])];
            let result = detect_keyword_rules("何かテキスト", &rules);
            assert!(result.is_empty());
        }

        #[test]
        fn k07_empty_inner_group_skipped() {
            let rules = vec![make_keyword_rule(
                "01H00000000000000000000024",
                vec![vec![]],
            )];
            let result = detect_keyword_rules("何かテキスト", &rules);
            assert!(
                result.is_empty(),
                "Empty inner group should not match, got: {:?}",
                result
            );
        }
    }

    mod detect_grammar_rules_in_text {
        use super::*;

        fn all_test_rules() -> Vec<GrammarRule> {
            vec![
                make_rule(
                    "01H00000000000000000000001",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]),
                ),
                make_rule(
                    "01H00000000000000000000002",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTa {}])]),
                ),
                make_rule(
                    "01H00000000000000000000003",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToMasu {}])]),
                ),
                make_rule(
                    "01H00000000000000000000004",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToNai {}])]),
                ),
                make_rule(
                    "01H00000000000000000000005",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTeForm {},
                            FormatAction::AddPostfix {
                                postfix: "ください".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000006",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTeForm {},
                            FormatAction::AddPostfix {
                                postfix: "います".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000007",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToStem {},
                            FormatAction::AddPostfix {
                                postfix: "ながら".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000008",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToNai {},
                            FormatAction::AddPostfix {
                                postfix: "ければなりません".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000009",
                    HashMap::from([(
                        PartOfSpeech::Verb,
                        vec![
                            FormatAction::VerbToTai {},
                            FormatAction::AddPostfix {
                                postfix: "です".to_string(),
                            },
                        ],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000010",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AddPostfix {
                            postfix: "です".to_string(),
                        }],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000011",
                    HashMap::from([(
                        PartOfSpeech::NaAdjective,
                        vec![FormatAction::AddPostfix {
                            postfix: "です".to_string(),
                        }],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000012",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AdjectiveToSugiru {}],
                    )]),
                ),
                make_rule(
                    "01H00000000000000000000013",
                    HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToPotential {}])]),
                ),
                make_rule(
                    "01H00000000000000000000014",
                    HashMap::from([(
                        PartOfSpeech::IAdjective,
                        vec![FormatAction::AdjectiveToKute {}],
                    )]),
                ),
            ]
        }

        #[test]
        fn e01_empty_tokens_returns_empty() {
            let rules = all_test_rules();
            let result = detect_grammar_rules_in_text("", &[], &rules);
            assert!(result.is_empty());
        }

        #[test]
        fn e02_particle_not_vocabulary() {
            let rules = all_test_rules();
            let tokens = vec![make_particle_token("は")];
            let result = detect_grammar_rules_in_text("は", &tokens, &rules);
            assert!(result.is_empty());
        }

        #[test]
        fn e03_combined_format_and_keyword_deduped() {
            let fm_rule = make_rule(
                "01H00000000000000000000030",
                HashMap::from([(PartOfSpeech::Verb, vec![FormatAction::VerbToTeForm {}])]),
            );
            let kw_rule = make_keyword_rule(
                "01H00000000000000000000031",
                vec![vec!["ながら".to_string()]],
            );
            let rules = vec![fm_rule, kw_rule];

            let tokens = vec![make_verb_token("食べる")];
            let result = detect_grammar_rules_in_text("食べてながら", &tokens, &rules);
            assert_eq!(
                result.len(),
                2,
                "Should detect both format_map te-form and keyword ながら, got: {:?}",
                result
            );
        }
    }
}
