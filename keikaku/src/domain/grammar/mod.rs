pub mod adjective_naru;
pub mod adjective_past;
pub mod nda;
pub mod verb_forms;
pub mod verb_hou_ga_ii;
pub mod verb_mada_te_inai;
pub mod verb_masenka;
pub mod verb_mashou;
pub mod verb_mashouka;
pub mod verb_naide_kudasai;
pub mod verb_ni_iku;
pub mod verb_sugiru;
pub mod verb_ta_koto_ga_aru;
pub mod verb_tai;
pub mod verb_tari;
pub mod verb_te_iru;
pub mod verb_te_kudasai;
pub mod verb_te_wa_ikemasen;
pub mod verb_tsumori;

use std::sync::LazyLock;

use crate::domain::{
    KeikakuError,
    grammar::{
        adjective_naru::AdjectiveNaruRule, adjective_past::AdjectivePastRule, nda::んだRule,
        verb_hou_ga_ii::VerbHouGaIiRule, verb_mada_te_inai::VerbMadaTeInaiRule,
        verb_masenka::VerbMasenkaRule, verb_mashou::VerbMashouRule,
        verb_mashouka::VerbMashoukaRule, verb_naide_kudasai::VerbNaideKudasaiRule,
        verb_ni_iku::VerbNiIkuRule, verb_sugiru::VerbSugiruRule,
        verb_ta_koto_ga_aru::VerbTaKotoGaAruRule, verb_tai::VerbTaiRule, verb_tari::VerbTariRule,
        verb_te_iru::VerbTeIruRule, verb_te_kudasai::VerbTeKudasaiRule,
        verb_te_wa_ikemasen::VerbTeWaIkemasenRule, verb_tsumori::ConstructionTsumoriRule,
    },
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

static GRAMMAR_LIST: LazyLock<Vec<Box<dyn GrammarRule>>> = LazyLock::new(|| {
    vec![
        // Существующие правила
        Box::new(んだRule {}),
        Box::new(AdjectivePastRule {}),
        // Глагольные формы
        Box::new(VerbMasenkaRule {}),
        Box::new(VerbMashouRule {}),
        Box::new(VerbMashoukaRule {}),
        Box::new(VerbTeKudasaiRule {}),
        Box::new(VerbTeWaIkemasenRule {}),
        Box::new(VerbTeIruRule {}),
        Box::new(VerbNiIkuRule {}),
        Box::new(VerbNaideKudasaiRule {}),
        Box::new(VerbMadaTeInaiRule {}),
        Box::new(VerbTaiRule {}),
        Box::new(VerbTariRule {}),
        Box::new(VerbTaKotoGaAruRule {}),
        Box::new(VerbSugiruRule {}),
        Box::new(VerbHouGaIiRule {}),
        // Конструкции с прилагательными
        Box::new(AdjectiveNaruRule {}),
        // Конструкции намерения
        Box::new(ConstructionTsumoriRule {}),
    ]
});

pub fn grammar_rules() -> &'static [Box<dyn GrammarRule>] {
    &GRAMMAR_LIST
}

pub trait GrammarRule: Send + Sync {
    fn level(&self) -> JapaneseLevel;
    fn title(&self, lang: &NativeLanguage) -> String;
    fn md_description(&self, lang: &NativeLanguage) -> String;

    fn apply_to(&self) -> Vec<PartOfSpeech>;
    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError>;
}
