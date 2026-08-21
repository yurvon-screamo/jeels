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
