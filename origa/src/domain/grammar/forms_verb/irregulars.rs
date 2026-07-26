use super::classify::{VerbGroup, classify_verb};
use super::godan_tables::GODAN_TO_STEM;
use super::te_ta::to_te_form;

#[derive(Clone, Copy)]
pub struct IrregularMapping {
    pub suru: &'static str,
    pub kuru: &'static str,
}

impl IrregularMapping {
    pub const fn new(suru: &'static str, kuru: &'static str) -> Self {
        Self { suru, kuru }
    }
}

pub const IRREGULAR_SURU: &str = "する";
pub const IRREGULAR_KURU: &str = "くる";

pub const MAIN_VIEW_IRREGULAR: IrregularMapping = IrregularMapping::new("し", "き");
pub const NAI_IRREGULAR: IrregularMapping = IrregularMapping::new("しない", "こない");
pub const TARA_IRREGULAR: IrregularMapping = IrregularMapping::new("したら", "きたら");
pub const BA_IRREGULAR: IrregularMapping = IrregularMapping::new("すれば", "くれば");
pub const POTENTIAL_IRREGULAR: IrregularMapping = IrregularMapping::new("できる", "こられる");
pub const PASSIVE_IRREGULAR: IrregularMapping = IrregularMapping::new("される", "こられる");
pub const CAUSATIVE_IRREGULAR: IrregularMapping = IrregularMapping::new("させる", "こさせる");
pub const CAUSATIVE_PASSIVE_IRREGULAR: IrregularMapping =
    IrregularMapping::new("させられる", "こさせられる");
pub const IMPERATIVE_IRREGULAR: IrregularMapping = IrregularMapping::new("しろ", "こい");
pub const VOLITIONAL_IRREGULAR: IrregularMapping = IrregularMapping::new("しよう", "こよう");
pub const ZU_IRREGULAR: IrregularMapping = IrregularMapping::new("せず", "こず");
pub const MIZENKEI_IRREGULAR: IrregularMapping = IrregularMapping::new("せ", "こ");
pub const O_NI_NARIMASU_IRREGULAR: IrregularMapping =
    IrregularMapping::new("なさる", "いらっしゃる");
pub const O_KUDASAI_IRREGULAR: IrregularMapping =
    IrregularMapping::new("なさってください", "いらっしゃってください");
pub const O_SHIMASU_IRREGULAR: IrregularMapping = IrregularMapping::new("いたす", "参る");

pub fn get_irregular_form(verb: &str, mapping: IrregularMapping) -> Option<String> {
    match verb {
        IRREGULAR_SURU => Some(mapping.suru.to_string()),
        IRREGULAR_KURU | "来る" => Some(mapping.kuru.to_string()),
        _ => None,
    }
}

pub fn stem_from_godan(word: &str) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let last_char = chars[chars.len() - 1];
    for (from, to) in GODAN_TO_STEM {
        if last_char == *from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return Some(result);
        }
    }
    None
}

pub fn is_ichidan(word: &str) -> bool {
    classify_verb(word) == VerbGroup::Ichidan
}

pub fn te_form_stem(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        te.strip_suffix('て').unwrap_or(&te).to_string()
    } else if te.ends_with("で") {
        te.strip_suffix('で').unwrap_or(&te).to_string()
    } else {
        te
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_view_converts_suru_to_shi() {
        assert_eq!(
            get_irregular_form("する", MAIN_VIEW_IRREGULAR),
            Some("し".to_string())
        );
    }

    #[test]
    fn main_view_converts_kuru_to_ki() {
        assert_eq!(
            get_irregular_form("くる", MAIN_VIEW_IRREGULAR),
            Some("き".to_string())
        );
    }

    #[test]
    fn regular_verb_returns_none() {
        assert_eq!(get_irregular_form("食べる", NAI_IRREGULAR), None);
    }
}
