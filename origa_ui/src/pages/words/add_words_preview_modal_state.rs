use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::{
    AnalyzeTextForCardsUseCase, AnalyzedWord, CreateCardsFromAnalysisResult,
    CreateCardsFromAnalysisUseCase, WordToCreate,
};
use std::collections::HashSet;
use tracing::{error, info};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Text,
    Anki,
    Image,
    Audio,
}

/// Pure view-stage decision extracted from the component for unit testing.
/// Determines which content the add-words modal renders, without needing
/// reactive signals or DOM.
///
/// Precedence: analyzing > preview (has words) > input (initial/reset/empty).
///
/// The former `NoResults` variant was removed: when analysis finds 0 words the
/// modal falls back to `Input` so the user can try a different file or method
/// instead of being stuck on a dead-end screen. The informational notice is
/// driven separately by `has_analyzed && analyzed_words.is_empty()` in the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStage {
    /// No analysis performed yet, reset, or analysis completed with 0 words —
    /// show input tabs (optionally with a "no words found" notice).
    Input,
    /// Analysis in progress — spinner / disabled state.
    Analyzing,
    /// Analysis completed with results — show preview list.
    Preview,
}

pub fn analysis_stage(words_count: usize, is_analyzing: bool) -> AnalysisStage {
    if is_analyzing {
        AnalysisStage::Analyzing
    } else if words_count > 0 {
        AnalysisStage::Preview
    } else {
        AnalysisStage::Input
    }
}

#[derive(Clone)]
pub struct PreviewModalState {
    pub input_mode: RwSignal<InputMode>,
    pub active_tab: RwSignal<String>,
    pub input_text: RwSignal<String>,
    pub analyzed_words: RwSignal<Vec<AnalyzedWord>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_analyzing: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    /// True once an analysis has finished (success with 0+ words, or the user
    /// has run at least one `analyze_text`). Reset to false on `reset()`.
    /// Drives the "no words found" notice in the Input stage.
    pub has_analyzed: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub refresh_trigger: RwSignal<u32>,
    pub disposed: StoredValue<()>,
}

impl PreviewModalState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let selected_words = RwSignal::new(HashSet::new());
        let disposed = StoredValue::new(());

        Effect::new({
            let selected_words_clone = selected_words;
            move |_| {
                if is_open.get() {
                    selected_words_clone.set(HashSet::new());
                }
            }
        });

        Self {
            input_mode: RwSignal::new(InputMode::Text),
            active_tab: RwSignal::new("text".to_string()),
            input_text: RwSignal::new(String::new()),
            analyzed_words: RwSignal::new(Vec::new()),
            selected_words,
            is_analyzing: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            has_analyzed: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            refresh_trigger,
            disposed,
        }
    }

    pub fn analyze_text(&self) {
        let text = self.input_text.get_untracked();
        let repository = self.repository.clone();
        let analyzed_words = self.analyzed_words;
        let selected_words = self.selected_words;
        let is_analyzing = self.is_analyzing;
        let has_analyzed = self.has_analyzed;
        let error = self.error_message;
        let disposed = self.disposed;

        is_analyzing.set(true);
        error.set(None);

        info!(text_length = text.len(), "Starting text analysis");

        spawn_local(async move {
            let use_case = AnalyzeTextForCardsUseCase::new(&repository);
            match use_case.execute(text).await {
                Ok(result) => {
                    info!(word_count = result.words.len(), "Text analysis completed");
                    if disposed.is_disposed() {
                        return;
                    }
                    let words_to_select: HashSet<String> = result
                        .words
                        .iter()
                        .filter(|w| w.meaning.is_some())
                        .map(|w| w.base_form.clone())
                        .collect();
                    analyzed_words.set(result.words);
                    selected_words.set(words_to_select);
                    is_analyzing.set(false);
                    has_analyzed.set(true);
                },
                Err(e) => {
                    error!(error = %e, "Text analysis failed");
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(e.to_string()));
                    is_analyzing.set(false);
                },
            }
        });
    }

    pub fn set_extracted_text(&self, text: String) {
        self.input_text.set(text);
        self.analyze_text();
    }

    pub fn reset(&self) {
        self.input_mode.set(InputMode::Text);
        self.active_tab.set("text".to_string());
        self.input_text.set(String::new());
        self.analyzed_words.set(Vec::new());
        self.selected_words.set(HashSet::new());
        self.has_analyzed.set(false);
        self.error_message.set(None);
    }

    pub fn toggle_word(&self, word: String) {
        self.selected_words.update(|selected| {
            if selected.contains(&word) {
                selected.remove(&word);
            } else {
                selected.insert(word);
            }
        });
    }

    pub fn create_cards(
        &self,
    ) -> impl Future<Output = Result<CreateCardsFromAnalysisResult, String>> {
        let selected_words = self.selected_words.get_untracked();
        let words_to_create: Vec<WordToCreate> = selected_words
            .into_iter()
            .map(|base_form| WordToCreate { base_form })
            .collect();
        let repository = self.repository.clone();
        let is_creating = self.is_creating;
        let error = self.error_message;

        async move {
            is_creating.set(true);
            error.set(None);

            let use_case = CreateCardsFromAnalysisUseCase::new(&repository);
            match use_case.execute(words_to_create, None).await {
                Ok(result) => {
                    is_creating.set(false);
                    Ok(result)
                },
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_creating.set(false);
                    Err(e.to_string())
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::initial_input(0, false, AnalysisStage::Input)]
    #[case::analyzing(0, true, AnalysisStage::Analyzing)]
    #[case::analyzing_with_prior_words(2, true, AnalysisStage::Analyzing)]
    #[case::empty_results_fall_back_to_input(0, false, AnalysisStage::Input)]
    #[case::preview_with_words(3, false, AnalysisStage::Preview)]
    fn analysis_stage_decision(
        #[case] words_count: usize,
        #[case] is_analyzing: bool,
        #[case] expected: AnalysisStage,
    ) {
        assert_eq!(analysis_stage(words_count, is_analyzing), expected);
    }
}
