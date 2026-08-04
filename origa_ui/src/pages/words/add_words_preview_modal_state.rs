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
/// Precedence: analyzing > preview (has words) > no-results (analyzed, empty)
/// > input (initial/reset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStage {
    /// No analysis performed yet, or reset — show input tabs.
    Input,
    /// Analysis in progress — spinner / disabled state.
    Analyzing,
    /// Analysis completed but found 0 words — show feedback message.
    NoResults,
    /// Analysis completed with results — show preview list.
    Preview,
}

pub fn analysis_stage(words_count: usize, has_analyzed: bool, is_analyzing: bool) -> AnalysisStage {
    if is_analyzing {
        AnalysisStage::Analyzing
    } else if words_count > 0 {
        AnalysisStage::Preview
    } else if has_analyzed {
        AnalysisStage::NoResults
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
    /// Drives the NoResults feedback branch in the modal.
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
                    let words_to_select: HashSet<String> =
                        result.words.iter().map(|w| w.base_form.clone()).collect();
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
    #[case::initial_input(0, false, false, AnalysisStage::Input)]
    #[case::analyzing(0, false, true, AnalysisStage::Analyzing)]
    #[case::analyzing_with_prior_words(2, true, true, AnalysisStage::Analyzing)]
    #[case::no_results_after_analysis(0, true, false, AnalysisStage::NoResults)]
    #[case::preview_with_words(3, true, false, AnalysisStage::Preview)]
    #[case::preview_ignores_has_analyzed(1, false, false, AnalysisStage::Preview)]
    fn analysis_stage_decision(
        #[case] words_count: usize,
        #[case] has_analyzed: bool,
        #[case] is_analyzing: bool,
        #[case] expected: AnalysisStage,
    ) {
        assert_eq!(
            analysis_stage(words_count, has_analyzed, is_analyzing),
            expected
        );
    }
}
