use std::str::FromStr;

/// Name of the cookie that records a visitor's chosen UI locale on the landing
/// site. Shared between the `negotiate_locale` middleware (reads it) and the
/// language switcher in `Layout` (writes it client-side); keeping it a single
/// constant prevents the two sides from drifting apart silently.
pub const LOCALE_COOKIE: &str = "origa_locale";

/// Lifetime of [`LOCALE_COOKIE`], in seconds (HTTP `Max-Age` unit). One year,
/// matching the persistent locale cookie of the CSR app.
pub const LOCALE_COOKIE_MAX_AGE_SECS: u64 = 365 * 24 * 60 * 60;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Locale {
    En,
    Ru,
    Ko,
    Vi,
}

impl Locale {
    pub const ALL: &[Locale] = &[Locale::En, Locale::Ru, Locale::Ko, Locale::Vi];

    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
            Locale::Ko => "ko",
            Locale::Vi => "vi",
        }
    }

    pub fn og_locale(&self) -> &'static str {
        match self {
            Locale::En => "en_US",
            Locale::Ru => "ru_RU",
            Locale::Ko => "ko_KR",
            Locale::Vi => "vi_VN",
        }
    }

    pub fn path_prefix(&self) -> &'static str {
        match self {
            Locale::En => "",
            Locale::Ru => "/ru",
            Locale::Ko => "/ko",
            Locale::Vi => "/vi",
        }
    }

    pub fn content(&self) -> &'static Content {
        match self {
            Locale::En => &super::content::en::CONTENT,
            Locale::Ru => &super::content::ru::CONTENT,
            Locale::Ko => &super::content::ko::CONTENT,
            Locale::Vi => &super::content::vi::CONTENT,
        }
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Locale::Ko | Locale::Vi)
    }

    pub fn image_prefix(&self) -> &'static str {
        match self {
            Locale::Ko | Locale::Vi => "en",
            _ => self.as_str(),
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Locale::En => "EN",
            Locale::Ru => "RU",
            Locale::Ko => "KO",
            Locale::Vi => "VI",
        }
    }
}

impl FromStr for Locale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            "ru" => Ok(Self::Ru),
            "ko" => Ok(Self::Ko),
            "vi" => Ok(Self::Vi),
            _ => Err(()),
        }
    }
}

pub struct Content {
    // Meta
    pub site_name: &'static str,
    pub keywords: &'static str,
    pub breadcrumb_home: &'static str,
    // LearningResource `teaches` (free-text, localised per ADR-018)
    pub learning_resource_teaches_vocab: &'static str,
    pub learning_resource_teaches_kanji: &'static str,
    pub learning_resource_teaches_grammar: &'static str,
    pub learning_resource_teaches_listening: &'static str,

    // Header
    pub header_features: &'static str,
    pub header_compare: &'static str,
    pub header_download: &'static str,
    pub header_integrations: &'static str,
    pub header_blog: &'static str,
    pub header_docs: &'static str,

    // Blog index page (/blog)
    pub blog_index_meta_title: &'static str,
    pub blog_index_meta_description: &'static str,
    pub blog_index_h1: &'static str,
    pub blog_index_subtitle: &'static str,
    pub blog_index_empty: &'static str,
    pub blog_updated_label: &'static str,
    pub breadcrumb_blog: &'static str,
    pub breadcrumb_docs: &'static str,
    pub docs_updated_label: &'static str,

    // Footer
    pub footer_product: &'static str,
    pub footer_resources: &'static str,
    pub footer_license: &'static str,

    // Homepage
    pub home_meta_title: &'static str,
    pub home_meta_description: &'static str,
    pub home_schema_feature_list: &'static str,
    pub home_hero_title: &'static str,
    pub home_hero_subtitle: &'static str,
    pub home_cta_primary: &'static str,
    pub home_cta_secondary: &'static str,
    pub home_hero_tagline: &'static str,
    // Single-line stat strip under the hero (platforms, corpus size, levels)
    pub home_stats_line: &'static str,
    // Homepage mini-FAQ "free?" pair; the other two pairs reuse the features
    // FAQ strings (offline + interface languages) so copy stays in sync.
    pub home_faq_free_question: &'static str,
    pub home_faq_free_answer: &'static str,
    pub home_problem_h2: &'static str,
    pub home_problem_text: &'static str,
    pub home_features_h2: &'static str,
    pub home_feature_vocab_title: &'static str,
    pub home_feature_vocab_text: &'static str,
    pub home_feature_kanji_title: &'static str,
    pub home_feature_kanji_text: &'static str,
    pub home_feature_grammar_title: &'static str,
    pub home_feature_grammar_text: &'static str,
    pub home_feature_listening_title: &'static str,
    pub home_feature_listening_text: &'static str,
    pub home_principle_content_title: &'static str,
    pub home_principle_content_text: &'static str,
    pub home_principle_fsrs_title: &'static str,
    pub home_principle_fsrs_text: &'static str,
    pub home_principle_local_title: &'static str,
    pub home_principle_local_text: &'static str,
    pub home_principle_offline_title: &'static str,
    pub home_principle_offline_text: &'static str,
    pub home_cta_title: &'static str,

    // Features page
    pub features_meta_title: &'static str,
    pub features_meta_description: &'static str,
    pub features_schema_how_to_name: &'static str,
    pub features_h1: &'static str,
    pub features_vocab_h2: &'static str,
    pub features_vocab_label: &'static str,
    pub features_vocab_step1: &'static str,
    pub features_vocab_step2: &'static str,
    pub features_vocab_step3: &'static str,
    pub features_vocab_dict: &'static str,
    pub features_vocab_cards: &'static str,
    pub features_vocab_audio: &'static str,
    pub features_vocab_import: &'static str,
    pub features_vocab_fsrs: &'static str,
    pub features_vocab_jlpt: &'static str,
    pub features_kanji_h2: &'static str,
    pub features_kanji_subtitle: &'static str,
    pub features_kanji_furigana: &'static str,
    pub features_kanji_furigana_desc: &'static str,
    pub features_kanji_writing: &'static str,
    pub features_kanji_writing_desc: &'static str,
    pub features_kanji_dict: &'static str,
    pub features_kanji_dict_desc: &'static str,
    pub features_kanji_tests: &'static str,
    pub features_kanji_tests_desc: &'static str,
    pub features_kanji_insight: &'static str,
    pub features_grammar_h2: &'static str,
    pub features_grammar_subtitle: &'static str,
    pub features_grammar_jlpt: &'static str,
    pub features_grammar_jlpt_desc: &'static str,
    pub features_grammar_context: &'static str,
    pub features_grammar_context_desc: &'static str,
    pub features_grammar_tests: &'static str,
    pub features_grammar_tests_desc: &'static str,
    pub features_grammar_search: &'static str,
    pub features_grammar_search_desc: &'static str,
    pub features_grammar_insight: &'static str,
    pub features_listening_h2: &'static str,
    pub features_listening_subtitle: &'static str,
    pub features_listening_n1: &'static str,
    pub features_listening_n1_desc: &'static str,
    pub features_listening_audio: &'static str,
    pub features_listening_audio_desc: &'static str,
    pub features_listening_comp: &'static str,
    pub features_listening_comp_desc: &'static str,
    pub features_listening_everyday: &'static str,
    pub features_listening_everyday_desc: &'static str,
    pub features_listening_insight: &'static str,
    pub features_cta: &'static str,
    pub features_faq_h2: &'static str,
    pub faq_q1: &'static str,
    pub faq_a1: &'static str,
    pub faq_q2: &'static str,
    pub faq_a2: &'static str,
    pub faq_q3: &'static str,
    pub faq_a3: &'static str,
    pub faq_q4: &'static str,
    pub faq_a4: &'static str,
    pub faq_q5: &'static str,
    pub faq_a5: &'static str,

    // Compare page
    pub compare_meta_title: &'static str,
    pub compare_meta_description: &'static str,
    pub compare_h1: &'static str,
    pub compare_subtitle: &'static str,
    pub compare_table_feature: &'static str,
    pub compare_table_origa: &'static str,
    pub compare_table_anki: &'static str,
    pub compare_table_wanikani: &'static str,
    pub compare_table_bunpro: &'static str,
    pub compare_table_duolingo: &'static str,
    pub compare_vocab: &'static str,
    pub compare_kanji: &'static str,
    pub compare_grammar: &'static str,
    pub compare_listening: &'static str,
    pub compare_languages: &'static str,
    pub compare_offline: &'static str,
    pub compare_yes: &'static str,
    pub compare_partial: &'static str,
    pub compare_no: &'static str,
    pub compare_english_only: &'static str,
    pub compare_limited: &'static str,
    pub compare_manual: &'static str,
    pub compare_legend_title: &'static str,
    pub compare_legend_origa: &'static str,
    pub compare_legend_supported: &'static str,
    pub compare_legend_partial: &'static str,
    pub compare_legend_not_supported: &'static str,
    pub compare_legend_value: &'static str,
    pub compare_label_best_for: &'static str,
    pub compare_label_origa_wins: &'static str,
    pub compare_label_together: &'static str,
    // Anki
    pub compare_anki_title: &'static str,
    pub compare_anki_what: &'static str,
    pub compare_anki_when: &'static str,
    pub compare_anki_better: &'static str,
    pub compare_anki_together: &'static str,
    // WaniKani
    pub compare_wanikani_title: &'static str,
    pub compare_wanikani_what: &'static str,
    pub compare_wanikani_when: &'static str,
    pub compare_wanikani_better: &'static str,
    pub compare_wanikani_together: &'static str,
    // Bunpro
    pub compare_bunpro_title: &'static str,
    pub compare_bunpro_what: &'static str,
    pub compare_bunpro_when: &'static str,
    pub compare_bunpro_better: &'static str,
    pub compare_bunpro_together: &'static str,
    // Duolingo
    pub compare_duolingo_title: &'static str,
    pub compare_duolingo_subtitle: &'static str,
    pub compare_duolingo_start: &'static str,
    pub compare_duolingo_grow: &'static str,
    pub compare_duolingo_together: &'static str,
    pub compare_bridge_label: &'static str,
    pub compare_score: &'static str,

    // Download page
    pub download_meta_title: &'static str,
    pub download_meta_description: &'static str,
    pub download_h1: &'static str,
    pub download_windows: &'static str,
    pub download_windows_formats: &'static str,
    pub download_linux: &'static str,
    pub download_linux_formats: &'static str,
    pub download_macos: &'static str,
    pub download_macos_formats: &'static str,
    pub download_android: &'static str,
    pub download_android_formats: &'static str,
    pub download_web: &'static str,
    pub download_ios: &'static str,
    pub download_ios_formats: &'static str,
    pub download_ios_coming_soon: &'static str,
    pub download_button: &'static str,
    pub download_subtitle: &'static str,
    pub download_try_web: &'static str,

    // Integrations page
    pub integrations_meta_title: &'static str,
    pub integrations_meta_description: &'static str,
    pub integrations_h1: &'static str,
    pub integrations_subtitle: &'static str,
    pub integrations_hero_stat: &'static str,

    // Section headers
    pub integrations_section_exams: &'static str,
    pub integrations_section_textbooks: &'static str,
    pub integrations_section_apps: &'static str,

    // Tags
    pub integrations_tag_exam: &'static str,
    pub integrations_tag_app: &'static str,
    pub integrations_tag_textbook: &'static str,
    pub integrations_tag_content: &'static str,
    pub integrations_tag_import: &'static str,

    // JLPT card
    pub integrations_jlpt_name: &'static str,
    pub integrations_jlpt_desc: &'static str,
    pub integrations_jlpt_detail: &'static str,

    // Duolingo card
    pub integrations_duolingo_name: &'static str,
    pub integrations_duolingo_desc: &'static str,
    pub integrations_duolingo_detail: &'static str,

    // Minna no Nihongo card
    pub integrations_minna_name: &'static str,
    pub integrations_minna_desc: &'static str,
    pub integrations_minna_detail: &'static str,

    // Irodori card
    pub integrations_irodori_name: &'static str,
    pub integrations_irodori_desc: &'static str,
    pub integrations_irodori_detail: &'static str,

    // Migii card
    pub integrations_migii_name: &'static str,
    pub integrations_migii_desc: &'static str,
    pub integrations_migii_detail: &'static str,

    // Spy x Family card
    pub integrations_spy_name: &'static str,
    pub integrations_spy_desc: &'static str,
    pub integrations_spy_detail: &'static str,

    // Anki card
    pub integrations_anki_name: &'static str,
    pub integrations_anki_desc: &'static str,
    pub integrations_anki_detail: &'static str,
    pub integrations_anki_note: &'static str,

    // WIP banner (empty for stable locales)
    pub banner_wip: &'static str,

    // Footer Legal column + shared link labels (reused by the in-app links)
    pub footer_legal: &'static str,
    pub legal_privacy_link: &'static str,
    pub legal_terms_link: &'static str,

    // Privacy Policy page (/privacy)
    pub privacy_meta_title: &'static str,
    pub privacy_meta_description: &'static str,
    pub privacy_h1: &'static str,
    pub privacy_last_updated_label: &'static str,
    pub privacy_body: &'static str,

    // Terms of Service page (/terms)
    pub terms_meta_title: &'static str,
    pub terms_meta_description: &'static str,
    pub terms_h1: &'static str,
    pub terms_last_updated_label: &'static str,
    pub terms_body: &'static str,
}

mod en;
mod ko;
mod ru;
mod vi;
