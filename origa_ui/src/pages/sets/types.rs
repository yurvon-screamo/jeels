#[derive(Clone, PartialEq)]
pub struct PreviewWord {
    pub word: String,
    pub meaning: Option<String>,
    /// Import-path classification (see `WordImportOutcome`): drives the
    /// status icon, the tooltip, and whether the word is selectable.
    pub outcome: origa::domain::WordImportOutcome,
    pub set_id: String,
    pub set_title: String,
}

#[derive(Clone, PartialEq)]
pub struct SetInfo {
    pub set_id: String,
    pub title: String,
    pub description: String,
    pub word_count: Option<usize>,
    pub set_type: String,
    pub level: origa::domain::JapaneseLevel,
    pub is_imported: bool,
}

/// Projects a CDN set meta record into the page-level [`SetInfo`].
///
/// The visible title and description must follow the user's native language —
/// the meta record ships both `ru` and `en` variants.
pub fn set_info_from_meta(
    meta: &origa::domain::WellKnownSetMeta,
    is_imported: bool,
    lang: &origa::domain::NativeLanguage,
) -> SetInfo {
    SetInfo {
        set_id: meta.id.clone(),
        title: meta.title(lang).to_string(),
        description: meta.description(lang).to_string(),
        word_count: Some(meta.word_count),
        set_type: meta.set_type.clone(),
        level: meta.level,
        is_imported,
    }
}
