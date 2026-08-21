use crate::loaders::WellKnownSetLoaderImpl;
use crate::pages::sets::import_slice_budget::slice_end;
use crate::pages::sets::types::PreviewWord;
use crate::repository::HybridUserRepository;
use crate::utils::yield_to_browser;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{User, WordImportOutcome, WordImportPreview};
use origa::traits::UserRepository;
use origa::use_cases::{
    CreateCardsFromAnalysisResult, CreateCardsFromAnalysisUseCase, WordToCreate,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::future::Future;

#[derive(Clone)]
pub struct ImportPreviewModalState {
    pub preview_words: RwSignal<Vec<PreviewWord>>,
    pub selected_words: RwSignal<HashSet<String>>,
    pub is_loading_preview: RwSignal<bool>,
    pub is_importing: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub well_known_loader: WellKnownSetLoaderImpl,
    pub set_titles: RwSignal<HashMap<String, String>>,
    pub set_ids: RwSignal<Vec<String>>,
    pub disposed: StoredValue<()>,
}

impl ImportPreviewModalState {
    pub fn new() -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");
        let well_known_loader = WellKnownSetLoaderImpl::new();
        let disposed = StoredValue::new(());
        Self {
            preview_words: RwSignal::new(Vec::new()),
            selected_words: RwSignal::new(HashSet::new()),
            is_loading_preview: RwSignal::new(false),
            is_importing: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            well_known_loader,
            set_titles: RwSignal::new(HashMap::new()),
            set_ids: RwSignal::new(Vec::new()),
            disposed,
        }
    }

    pub fn load_preview(&self, set_id: String) {
        let repository = self.repository.clone();
        let well_known_loader = self.well_known_loader.clone();
        let preview_words = self.preview_words;
        let selected_words = self.selected_words;
        let is_loading_preview = self.is_loading_preview;
        let error = self.error_message;
        let set_titles = self.set_titles;
        let set_ids = self.set_ids;
        let disposed = self.disposed;

        preview_words.set(Vec::new());
        selected_words.set(HashSet::new());
        is_loading_preview.set(true);
        error.set(None);
        spawn_local(async move {
            let user =
                match load_current_user(repository, error, is_loading_preview, disposed).await {
                    Some(u) => u,
                    None => return,
                };
            if disposed.is_disposed() {
                return;
            }
            let set = match well_known_loader.load_set(set_id.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    if !disposed.is_disposed() {
                        error.set(Some(e.to_string()));
                        is_loading_preview.set(false);
                    }
                    return;
                },
            };
            let set_title = set_ids
                .get()
                .first()
                .and_then(|id| set_titles.get().get(id).cloned())
                .unwrap_or_else(|| set_id.clone());
            let words_preview = build_set_preview_words(
                &user,
                &[(set_id, set_title, set.words().to_vec())],
                disposed,
            )
            .await;
            finalize_preview(
                words_preview,
                preview_words,
                selected_words,
                is_loading_preview,
                disposed,
            );
        });
    }

    pub fn load_preview_multiple(
        &self,
        set_ids: Vec<String>,
        set_titles_input: HashMap<String, String>,
    ) {
        let repository = self.repository.clone();
        let well_known_loader = self.well_known_loader.clone();
        let preview_words = self.preview_words;
        let selected_words = self.selected_words;
        let is_loading_preview = self.is_loading_preview;
        let error = self.error_message;
        let set_titles = self.set_titles;
        let state_set_ids = self.set_ids;
        let disposed = self.disposed;

        preview_words.set(Vec::new());
        selected_words.set(HashSet::new());
        is_loading_preview.set(true);
        error.set(None);
        set_titles.set(set_titles_input);
        state_set_ids.set(set_ids.clone());
        spawn_local(async move {
            let user =
                match load_current_user(repository, error, is_loading_preview, disposed).await {
                    Some(u) => u,
                    None => return,
                };
            if disposed.is_disposed() {
                return;
            }
            let loaded_sets = match well_known_loader.load_sets(set_ids.clone()).await {
                Ok(sets) => sets,
                Err(e) => {
                    if !disposed.is_disposed() {
                        error.set(Some(e.to_string()));
                        is_loading_preview.set(false);
                    }
                    return;
                },
            };
            let titles = set_titles.get();
            // Classify ONE concatenated word list so lemma duplicates
            // across sets share one seen-lemmas state with the import.
            let mut groups = Vec::with_capacity(loaded_sets.len());
            for (set_id, set) in loaded_sets {
                let set_title = titles
                    .get(&set_id)
                    .cloned()
                    .unwrap_or_else(|| set_id.clone());
                groups.push((set_id, set_title, set.words().to_vec()));
            }
            let words_preview = build_set_preview_words(&user, &groups, disposed).await;
            finalize_preview(
                words_preview,
                preview_words,
                selected_words,
                is_loading_preview,
                disposed,
            );
        });
    }

    pub fn reset(&self) {
        self.preview_words.set(Vec::new());
        self.selected_words.set(HashSet::new());
        self.error_message.set(None);
        self.set_titles.set(HashMap::new());
        self.set_ids.set(Vec::new());
    }

    pub fn toggle_word(&self, word: String) {
        self.selected_words.update(|selected| {
            if !selected.remove(&word) {
                selected.insert(word);
            }
        });
    }

    pub fn import_selected(
        &self,
    ) -> impl Future<Output = Result<CreateCardsFromAnalysisResult, String>> {
        let selected_words = self.selected_words.get();
        let preview_words = self.preview_words.get();
        let set_ids: Vec<String> = preview_words
            .iter()
            .filter(|pw| selected_words.contains(&pw.word))
            .map(|pw| pw.set_id.clone())
            .collect();
        let unique_set_ids: Vec<String> = set_ids
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let words_to_create: Vec<WordToCreate> = selected_words
            .into_iter()
            .map(|base_form| WordToCreate { base_form })
            .collect();
        let repository = self.repository.clone();
        let is_importing = self.is_importing;
        let error = self.error_message;
        let set_ids_opt = if unique_set_ids.is_empty() {
            None
        } else {
            Some(unique_set_ids)
        };
        async move {
            is_importing.set(true);
            error.set(None);
            let use_case = CreateCardsFromAnalysisUseCase::new(&repository);
            match use_case.execute(words_to_create, set_ids_opt).await {
                Ok(result) => {
                    is_importing.set(false);
                    Ok(result)
                },
                Err(e) => {
                    error.set(Some(e.to_string()));
                    is_importing.set(false);
                    Err(e.to_string())
                },
            }
        }
    }
}

async fn load_current_user(
    repository: HybridUserRepository,
    error: RwSignal<Option<String>>,
    is_loading_preview: RwSignal<bool>,
    disposed: StoredValue<()>,
) -> Option<origa::domain::User> {
    let fail = |msg: String| {
        if disposed.is_disposed() {
            return;
        }
        error.set(Some(msg));
        is_loading_preview.set(false);
    };
    match repository.get_current_user().await {
        Ok(Some(u)) => Some(u),
        Ok(None) => {
            fail(
                crate::i18n::use_i18n()
                    .get_keys()
                    .shared()
                    .user_not_found()
                    .inner()
                    .to_string(),
            );
            None
        },
        Err(e) => {
            fail(e.to_string());
            None
        },
    }
}

fn finalize_preview(
    words_preview: Vec<PreviewWord>,
    preview_words: RwSignal<Vec<PreviewWord>>,
    selected_words: RwSignal<HashSet<String>>,
    is_loading_preview: RwSignal<bool>,
    disposed: StoredValue<()>,
) {
    if disposed.is_disposed() {
        return;
    }
    // Words without a dictionary entry cannot be imported (the import
    // would fail them), so they start unselected.
    let words_to_select: HashSet<String> = words_preview
        .iter()
        .filter(|w| w.outcome != WordImportOutcome::NoDictionaryEntry)
        .map(|w| w.word.clone())
        .collect();
    preview_words.set(words_preview);
    selected_words.set(words_to_select);
    is_loading_preview.set(false);
}

/// Classifies every word through the domain import classifier
/// (tokenize → lemma → dictionary → duplicate check), yielding to the
/// browser between time-budgeted slices so a large multi-set preview
/// cannot freeze the UI. The per-load classifier state dies with this
/// call — reopening the drawer classifies against fresh card data.
async fn build_set_preview_words(
    user: &User,
    groups: &[(String, String, Vec<String>)],
    disposed: StoredValue<()>,
) -> Vec<PreviewWord> {
    let flat: Vec<&String> = groups
        .iter()
        .flat_map(|(_, _, words)| words.iter())
        .collect();
    let mut classifier = user.word_import_classifier();
    let mut classified: HashMap<String, WordImportPreview> = HashMap::with_capacity(flat.len());
    let now = performance_now_ms();
    let mut cursor = 0;

    while cursor < flat.len() {
        let end = slice_end(flat.len(), cursor, &now);
        for word in &flat[cursor..end] {
            let preview = classifier.classify(word);
            classified.insert((*word).clone(), preview);
        }
        cursor = end;
        if cursor < flat.len() {
            yield_to_browser().await;
            if disposed.is_disposed() {
                return Vec::new();
            }
        }
    }

    let mut words_preview = Vec::with_capacity(flat.len());
    for (set_id, set_title, words) in groups {
        for word in words {
            // Every word went through the loop above; classifying again on
            // a cache miss is idempotent (the classifier caches internally)
            // and keeps this path panic-free.
            let preview = classified
                .get(word)
                .cloned()
                .unwrap_or_else(|| classifier.classify(word));
            words_preview.push(PreviewWord {
                word: word.clone(),
                meaning: preview.meaning,
                outcome: preview.outcome,
                set_id: set_id.clone(),
                set_title: set_title.clone(),
            });
        }
    }
    words_preview
}

fn performance_now_ms() -> impl Fn() -> f64 {
    move || {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }
}
