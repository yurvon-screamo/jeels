use crate::i18n::{t, use_i18n};
use crate::pages::sets::set_word_item::SetWordItem;
use crate::pages::sets::types::PreviewWord;
use crate::pages::shared::LoadMoreButton;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Drawer, Spinner, Text, TextSize, ToastData,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{User, WordImportOutcome};
use origa::traits::UserRepository;
use std::collections::HashMap;

use super::import_set_preview_modal_handlers::create_import_preview_handlers;
use super::import_set_preview_modal_state::ImportPreviewModalState;

const PREVIEW_PAGE_SIZE: usize = 100;

/// Group headers distinguish sets in a multi-set import. In a single-set
/// import the drawer title already names the set and the "found N words"
/// line already counts it, so a group header would duplicate both.
fn should_render_group_headers(set_ids: &[String]) -> bool {
    set_ids.len() > 1
}

/// Distributes a global word budget across set groups, keeping each group's
/// original order. Groups are rendered whole up to the budget; the group that
/// crosses it is cut off at the remaining slots. An empty input yields no
/// groups regardless of the budget.
pub(crate) fn cap_groups(
    groups: HashMap<String, Vec<PreviewWord>>,
    limit: usize,
) -> Vec<(String, Vec<PreviewWord>)> {
    let mut ids: Vec<String> = groups.keys().cloned().collect();
    ids.sort();
    let mut out = Vec::with_capacity(ids.len());
    let mut remaining = limit;
    for id in ids {
        if remaining == 0 {
            break;
        }
        let mut words = groups.get(&id).cloned().unwrap_or_default();
        if words.len() > remaining {
            words.truncate(remaining);
        }
        remaining = remaining.saturating_sub(words.len());
        if !words.is_empty() {
            out.push((id, words));
        }
    }
    out
}

#[component]
pub fn ImportSetPreviewModal(
    is_open: RwSignal<bool>,
    set_ids: Signal<Vec<String>>,
    set_titles: Signal<HashMap<String, String>>,
    toasts: RwSignal<Vec<ToastData>>,
    on_import_result: Callback<Vec<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let repo_for_init = repository.clone();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                current_user.set(Some(user));
            }
        });
    });

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let state = ImportPreviewModalState::new();
    let handlers = create_import_preview_handlers(state.clone(), is_open, toasts, on_import_result);

    // iOS WKWebView jetsams the process around ~1.5 GB of linear memory.
    // Rendering every preview word at once (a level multi-select loads
    // 8000+ items, each a FuriganaText + MarkdownText + Checkbox + Tooltip
    // subtree) blew past that and killed the page to a black screen. The
    // preview is therefore paginated; the import itself still covers ALL
    // words (selected_words is untouched by pagination).
    let visible_words: RwSignal<usize> = RwSignal::new(PREVIEW_PAGE_SIZE);

    let preview_words = state.preview_words;
    let selected_words = state.selected_words;
    let is_loading_preview = state.is_loading_preview;
    let is_importing = state.is_importing;
    let error_message = state.error_message;

    let grouped_words = Memo::new(move |_| {
        let words = preview_words.get();
        let mut groups: HashMap<String, Vec<PreviewWord>> = HashMap::new();
        for word in words {
            groups.entry(word.set_id.clone()).or_default().push(word);
        }
        groups
    });

    let paginated_groups = Memo::new(move |_| {
        let groups = grouped_words.get();
        cap_groups(groups, visible_words.get())
    });

    let group_word_counts = Memo::new(move |_| {
        let groups = grouped_words.get();
        groups
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect::<HashMap<_, _>>()
    });

    let drawer_title = Memo::new(move |_| {
        let ids = set_ids.get();
        if ids.len() == 1 {
            set_titles
                .get()
                .get(&ids[0])
                .cloned()
                .unwrap_or_else(|| i18n.get_keys().sets().import_set().inner().to_string())
        } else {
            i18n.get_keys()
                .sets()
                .import_sets()
                .inner()
                .to_string()
                .replacen("{}", &ids.len().to_string(), 1)
        }
    });

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                visible_words.set(PREVIEW_PAGE_SIZE);
                let ids = set_ids.get();
                let titles = set_titles.get();
                if ids.len() == 1 {
                    state.load_preview(ids[0].clone());
                } else {
                    state.load_preview_multiple(ids, titles);
                }
            }
        }
    });

    let total_words_count = Memo::new(move |_| {
        let groups = grouped_words.get();
        groups.values().map(|g| g.len()).sum::<usize>()
    });

    // Summary buckets count UNIQUE words — the import processes a HashSet
    // of selected words, so a word listed in two sets counts once. The
    // buckets converge with the import toast: New → "created",
    // AlreadyExists + DuplicateInSelection → "skipped"; NoDictionaryEntry
    // words are unselectable and never reach the import.
    let import_breakdown = Memo::new(move |_| {
        let words = preview_words.get();
        let mut unique: HashMap<String, WordImportOutcome> = HashMap::new();
        for word in words {
            unique.entry(word.word.clone()).or_insert(word.outcome);
        }
        let new_count = unique
            .values()
            .filter(|o| **o == WordImportOutcome::New)
            .count();
        let existing_count = unique
            .values()
            .filter(|o| {
                matches!(
                    o,
                    WordImportOutcome::AlreadyExists | WordImportOutcome::DuplicateInSelection
                )
            })
            .count();
        let no_dictionary_count = unique
            .values()
            .filter(|o| **o == WordImportOutcome::NoDictionaryEntry)
            .count();
        (unique.len(), new_count, existing_count, no_dictionary_count)
    });

    // In a single-set import the drawer title already names the set and the
    // "found N words" line already counts it — a per-group header would
    // repeat both. Multi-set imports keep group headers to tell sets apart.
    let show_group_headers = Memo::new(move |_| should_render_group_headers(&set_ids.get()));

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(move || drawer_title.get())
            test_id="sets-import-drawer"
        >
            <div class="flex flex-col h-full">
                {move || {
                    let groups = grouped_words.get();
                    let is_loading = is_loading_preview.get();

                    if let Some(error) = error_message.get() {
                        view! {
                            <Alert
                                alert_type=Signal::derive(|| AlertType::Error)
                                title=Signal::derive(move || i18n.get_keys().common().error().inner().to_string())
                                message=Signal::derive(move || error.clone())
                            />
                        }.into_any()
                    } else if is_loading {
                        view! {
                            <div class="flex flex-col items-center py-4 gap-3">
                                <Spinner />
                                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                    {t!(i18n, sets.loading_words)}
                                </Text>
                            </div>
                        }.into_any()
                    } else if groups.is_empty() {
                        view! {
                            <div class="flex flex-col items-center py-4 gap-3" data-testid="sets-drawer-empty">
                                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                    {t!(i18n, sets.no_words)}
                                </Text>
                            </div>
                        }.into_any()
                    } else {
                        let titles_map = set_titles.get();
                        let kanji = known_kanji.get();
                        let selected = selected_words;
                        let with_group_headers = show_group_headers.get();
                        let shown_count = paginated_groups
                            .get()
                            .iter()
                            .map(|(_, g)| g.len())
                            .sum::<usize>();

                        view! {
                            <div class="mb-4" data-testid="sets-drawer-found">
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {i18n.get_keys().sets().found_words().inner().to_string()
                                        .replacen("{}", &import_breakdown.get().0.to_string(), 1)
                                        .replacen("{}", &import_breakdown.get().1.to_string(), 1)
                                        .replacen("{}", &import_breakdown.get().2.to_string(), 1)
                                        .replacen("{}", &import_breakdown.get().3.to_string(), 1)}
                                </Text>
                            </div>
                            <div class="space-y-6">
                                {paginated_groups
                                    .get()
                                    .into_iter()
                                    .map(|(set_id, words)| {
                                        let title = titles_map
                                            .get(&set_id)
                                            .cloned()
                                            .unwrap_or_else(|| set_id.clone());
                                        let word_count = group_word_counts.get().get(&set_id).copied().unwrap_or(0);

                                        // Plain conditional (not <Show>): the
                                        // whole preview block re-renders on
                                        // signal changes, so per-render state
                                        // is enough here.
                                        let group_header = with_group_headers.then(|| {
                                            view! {
                                                <h3 class="font-semibold text-base mb-3 text-gray-700">
                                                    {title}
                                                    <span class="text-gray-400 font-normal ml-2">
                                                        {i18n.get_keys().sets().words_count().inner().to_string().replacen("{}", &word_count.to_string(), 1)}
                                                    </span>
                                                </h3>
                                            }
                                        });

                                        view! {
                                            <div class="border-b border-gray-200 pb-4 last:border-0">
                                                {group_header}
                                                <div class="space-y-2">
                                                    {words
                                                        .into_iter()
                                                        .map(|word| {
                                                            let word_text = word.word.clone();
                                                            let known_meaning = word.meaning.clone();
                                                            let outcome = word.outcome;

                                                            view! {
                                                                <SetWordItem
                                                                    word=word_text.clone()
                                                                    known_meaning=known_meaning
                                                                    outcome=outcome
                                                                    selected_words=selected
                                                                    known_kanji=kanji.clone()
                                                                    on_toggle=Callback::new(move |_| handlers.on_word_toggle.run(word_text.clone()))
                                                                />
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                            <Show when=move || shown_count < total_words_count.get()>
                                <div class="pt-2 mt-4">
                                    <LoadMoreButton
                                        visible_count=visible_words
                                        total=Signal::derive(move || total_words_count.get())
                                        page_size=PREVIEW_PAGE_SIZE
                                        test_id=Signal::derive(|| "sets-drawer-load-more-btn".to_string())
                                    />
                                </div>
                            </Show>
                            <div class="sticky bottom-0 mt-4 pt-4 pb-2 border-t bg-[var(--bg-paper)] flex gap-2 justify-between">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    on_click=handlers.on_cancel
                                    test_id="sets-drawer-cancel-btn"
                                >
                                    {t!(i18n, common.cancel)}
                                </Button>
                                <Button
                                    variant=ButtonVariant::Olive
                                    disabled=Signal::derive(move || {
                                        selected_words.get().is_empty()
                                            || is_importing.get()
                                    })
                                    on_click=Callback::new(move |_| handlers.on_import.run(()))
                                    test_id="sets-drawer-import-btn"
                                >
                                    {move || {
                                        if is_importing.get() {
                                            t!(i18n, sets.importing).into_any()
                                        } else {
                                            t!(i18n, sets.import_button).into_any()
                                        }
                                    }}
                                </Button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </Drawer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(set: &str, idx: usize) -> PreviewWord {
        PreviewWord {
            word: format!("{set}-{idx}"),
            meaning: None,
            outcome: origa::domain::WordImportOutcome::New,
            set_id: set.to_string(),
            set_title: set.to_string(),
        }
    }

    // Single-set import: the group header would repeat the drawer title
    // and the word count.
    #[test]
    fn should_render_group_headers_single_set_is_false() {
        assert!(!should_render_group_headers(&["jlpt-n5".to_string()]));
    }

    #[test]
    fn should_render_group_headers_multi_set_is_true() {
        assert!(should_render_group_headers(&[
            "jlpt-n5".to_string(),
            "minna-1".to_string()
        ]));
    }

    #[test]
    fn cap_groups_limits_total_rendered_words() {
        let mut groups = HashMap::new();
        groups.insert("b".to_string(), (0..60).map(|i| word("b", i)).collect());
        groups.insert("a".to_string(), (0..80).map(|i| word("a", i)).collect());
        let capped = cap_groups(groups, 100);
        let total: usize = capped.iter().map(|(_, g)| g.len()).sum();
        assert_eq!(total, 100);
        // deterministic order: sorted ids first
        assert_eq!(capped[0].0, "a");
        assert_eq!(capped[0].1.len(), 80);
        assert_eq!(capped[1].0, "b");
        assert_eq!(capped[1].1.len(), 20);
    }

    #[test]
    fn cap_groups_zero_budget_renders_nothing() {
        let mut groups = HashMap::new();
        groups.insert("a".to_string(), vec![word("a", 0), word("a", 1)]);
        assert!(cap_groups(groups, 0).is_empty());
    }

    #[test]
    fn cap_groups_budget_above_total_keeps_everything() {
        let mut groups = HashMap::new();
        groups.insert("a".to_string(), vec![word("a", 0)]);
        groups.insert("b".to_string(), vec![word("b", 0), word("b", 1)]);
        let capped = cap_groups(groups, 100);
        assert_eq!(capped.len(), 2);
        assert_eq!(capped.iter().map(|(_, g)| g.len()).sum::<usize>(), 3);
    }
}
