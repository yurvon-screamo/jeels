use super::filters::{ImportFilter, LevelFilter, TypeFilter, available_set_types};
use super::import_set_preview_modal::ImportSetPreviewModal;
use super::sets_level_group::SetsLevelGroup;
use super::types::SetInfo;
use crate::i18n::{t, use_i18n};
use crate::loaders::WellKnownSetLoaderImpl;
use crate::pages::shared::LoadMoreButton;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, FilterTag, Input, Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::TypeMeta;
use origa::domain::{JapaneseLevel, User};
use origa::traits::UserRepository;
use std::collections::{HashMap, HashSet};

#[component]
pub fn SetsContent() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let sets: RwSignal<Vec<SetInfo>> = RwSignal::new(Vec::new());
    let is_loading: RwSignal<bool> = RwSignal::new(true);
    let load_error: RwSignal<bool> = RwSignal::new(false);
    let preview_modal_open = RwSignal::new(false);
    let preview_set_ids = RwSignal::new(Vec::<String>::new());
    let preview_set_titles = RwSignal::new(HashMap::<String, String>::new());
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let level_filter = RwSignal::new(LevelFilter::default());
    let type_filter = RwSignal::new(TypeFilter::default());
    let import_filter = RwSignal::new(ImportFilter::default());
    let search = RwSignal::new(String::new());
    let selected_sets: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let visible_count: RwSignal<usize> = RwSignal::new(50);
    let disposed = StoredValue::new(());

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let repo_for_init = repository.clone();
    let loader = WellKnownSetLoaderImpl::new();
    let sets_for_load = sets;
    let initialized = RwSignal::new(false);

    Effect::new(move |_| {
        if initialized.get() {
            return;
        }
        initialized.set(true);

        let repo = repo_for_init.clone();
        let loader = loader.clone();
        spawn_local(async move {
            let user = match repo.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    current_user.set(Some(user.clone()));
                    Some(user)
                },
                Ok(None) => {
                    tracing::warn!("SetsContent: user not found");
                    None
                },
                Err(e) => {
                    tracing::error!("SetsContent: get_current_user error: {:?}", e);
                    None
                },
            };

            match loader.load_meta_list().await {
                Ok(meta_list) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    load_error.set(false);
                    let set_list: Vec<SetInfo> = meta_list
                        .into_iter()
                        .map(|meta| {
                            let is_imported = user
                                .as_ref()
                                .map(|u| u.is_set_imported(&meta.id))
                                .unwrap_or(false);

                            SetInfo {
                                set_id: meta.id,
                                title: meta.title_ru,
                                description: meta.desc_ru,
                                word_count: Some(meta.word_count),
                                set_type: meta.set_type,
                                level: meta.level,
                                is_imported,
                            }
                        })
                        .collect();
                    sets_for_load.set(set_list);
                },
                Err(e) => {
                    tracing::error!("SetsContent: load_meta_list error: {:?}", e);
                    if !disposed.is_disposed() {
                        load_error.set(true);
                    }
                },
            }
            is_loading.set(false);
        });
    });

    let sets_for_update = sets;
    Effect::new(move |_| {
        let user = current_user.get();
        sets_for_update.update(|set_list| {
            if let Some(u) = user {
                for set in set_list.iter_mut() {
                    set.is_imported = u.is_set_imported(&set.set_id);
                }
            }
        });
    });

    let filtered_sets = Memo::new(move |_| {
        let level = level_filter.get();
        let type_f = type_filter.get();
        let import_f = import_filter.get();
        let query = search.get().to_lowercase();

        sets.get()
            .into_iter()
            .filter(|s| {
                let matches_level = level.matches(s.level);
                let matches_type = type_f.matches(&s.set_type);
                let matches_import = import_f.matches(s.is_imported);
                let matches_search = query.is_empty()
                    || s.title.to_lowercase().contains(&query)
                    || s.description.to_lowercase().contains(&query);
                matches_level && matches_type && matches_import && matches_search
            })
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        let _ = search.get();
        let _ = level_filter.get();
        let _ = type_filter.get();
        let _ = import_filter.get();
        let _ = sets.get();
        visible_count.set(50);
    });

    let visible_sets = Memo::new(move |_| {
        filtered_sets
            .get()
            .into_iter()
            .take(visible_count.get())
            .collect::<Vec<_>>()
    });

    let on_import = Callback::new(move |(set_id, title): (String, String)| {
        preview_set_ids.set(vec![set_id.clone()]);
        preview_set_titles.update(|m| {
            m.clear();
            m.insert(set_id, title);
        });
        preview_modal_open.set(true);
    });

    let on_toggle_select = Callback::new(move |set_id: String| {
        selected_sets.update(|sets| {
            if sets.contains(&set_id) {
                sets.remove(&set_id);
            } else {
                sets.insert(set_id);
            }
        });
    });

    let known_kanji_clone = known_kanji.get();

    view! {
        <div class="sets-page">
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4" data-testid="sets-loading">
                    <Spinner />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id="sets-loading-text">
                        {t!(i18n, sets.loading_sets)}
                    </Text>
                </div>
            </Show>
            <Show when=move || !is_loading.get() && load_error.get()>
                <div
                    class="flex flex-col items-center py-12 px-4 text-center gap-3"
                    data-testid="sets-offline-error"
                >
                    <Text size=TextSize::Large>
                        {t!(i18n, sets.offline_title)}
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, sets.offline_desc)}
                    </Text>
                </div>
            </Show>
            <Show when=move || !is_loading.get() && !load_error.get()>
                <div class="space-y-4 mb-6">
                    <Input
                        value=search
                        placeholder=Signal::derive(move || i18n.get_keys().sets().search_sets().inner().to_string())
                        test_id="sets-search-input"
                    />

                    <div class="flex flex-wrap gap-2" data-testid="sets-level-filters">
                        <FilterTag
                            label=LevelFilter::All.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::All)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::All);
                            })
                            test_id=Signal::derive(|| "sets-level-all".to_string())
                        />
                        <FilterTag
                            label=LevelFilter::N5.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::N5)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N5);
                            })
                            test_id=Signal::derive(|| "sets-level-n5".to_string())
                        />
                        <FilterTag
                            label=LevelFilter::N4.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::N4)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N4);
                            })
                            test_id=Signal::derive(|| "sets-level-n4".to_string())
                        />
                        <FilterTag
                            label=LevelFilter::N3.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::N3)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N3);
                            })
                            test_id=Signal::derive(|| "sets-level-n3".to_string())
                        />
                        <FilterTag
                            label=LevelFilter::N2.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::N2)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N2);
                            })
                            test_id=Signal::derive(|| "sets-level-n2".to_string())
                        />
                        <FilterTag
                            label=LevelFilter::N1.label(&i18n)
                            is_active=Signal::derive(move || level_filter.get() == LevelFilter::N1)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                level_filter.set(LevelFilter::N1);
                            })
                            test_id=Signal::derive(|| "sets-level-n1".to_string())
                        />
                    </div>

                    <div class="flex flex-wrap gap-2" data-testid="sets-type-filters">
                        <FilterTag
                            label=i18n.get_keys_untracked().sets().all_types().inner().to_string()
                            is_active=Signal::derive(move || type_filter.get().is_all())
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                type_filter.set(TypeFilter::all());
                            })
                            test_id=Signal::derive(|| "sets-type-all".to_string())
                        />
                        <For
                            each=move || available_set_types()
                            key=|type_meta: &TypeMeta| type_meta.id.clone()
                            children=move |type_meta| {
                                let type_id = type_meta.id.clone();
                                let type_id_for_click = type_id.clone();
                                let type_filter = type_filter;
                                let label = type_meta.label_ru.clone();
                                let test_id_val = format!("sets-type-{}", type_id);
                                view! {
                                    <FilterTag
                                        label=label.clone()
                                        is_active=Signal::derive(move || {
                                            matches!(&type_filter.get().0, Some(filter_id) if filter_id == &type_id)
                                        })
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            type_filter.set(TypeFilter::specific(&type_id_for_click));
                                        })
                                        test_id=Signal::derive(move || test_id_val.clone())
                                    />
                                }
                            }
                        />
                    </div>

                    <div class="flex flex-wrap gap-2" data-testid="sets-import-filters">
                        <FilterTag
                            label=ImportFilter::All.label(&i18n)
                            is_active=Signal::derive(move || import_filter.get() == ImportFilter::All)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::All);
                            })
                            test_id=Signal::derive(|| "sets-import-all".to_string())
                        />
                        <FilterTag
                            label=ImportFilter::Imported.label(&i18n)
                            is_active=Signal::derive(move || import_filter.get() == ImportFilter::Imported)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::Imported);
                            })
                            test_id=Signal::derive(|| "sets-import-imported".to_string())
                        />
                        <FilterTag
                            label=ImportFilter::New.label(&i18n)
                            is_active=Signal::derive(move || import_filter.get() == ImportFilter::New)
                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                import_filter.set(ImportFilter::New);
                            })
                            test_id=Signal::derive(|| "sets-import-new".to_string())
                        />
                    </div>

                    <Show when=move || !selected_sets.get().is_empty()>
                        <div class="flex items-center gap-3 pt-2">
                            <Button
                                variant=ButtonVariant::Olive
                                test_id="sets-import-selected-btn"
                                on_click=Callback::new(move |_| {
                                    let set_ids: Vec<String> = selected_sets.get().into_iter().collect();
                                    let count = set_ids.len();
                                    if count > 0 {
                                        let all_sets = sets.get();
                                        let titles: HashMap<String, String> = all_sets
                                            .iter()
                                            .filter(|s| set_ids.contains(&s.set_id))
                                            .map(|s| (s.set_id.clone(), s.title.clone()))
                                            .collect();

                                        preview_set_ids.set(set_ids);
                                        preview_set_titles.set(titles);
                                        preview_modal_open.set(true);
                                    }
                                })
                            >
                                {move || i18n.get_keys().sets().import_sets().inner().to_string().replacen("{}", &selected_sets.get().len().to_string(), 1)}
                            </Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                test_id="sets-cancel-select-btn"
                                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                    selected_sets.set(HashSet::new());
                                })
                            >
                                {t!(i18n, sets.cancel_selection)}
                            </Button>
                        </div>
                    </Show>
                </div>

                <SetsLevelGroup
                    level=JapaneseLevel::N5
                    sets=visible_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N4
                    sets=visible_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N3
                    sets=visible_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N2
                    sets=visible_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <SetsLevelGroup
                    level=JapaneseLevel::N1
                    sets=visible_sets
                    type_filter=type_filter
                    known_kanji=known_kanji_clone.clone()
                    on_import=on_import
                    selected_sets=selected_sets
                    on_toggle_select=on_toggle_select
                />
                <LoadMoreButton
                    visible_count=visible_count
                    total=Signal::derive(move || filtered_sets.get().len())
                    test_id=Signal::derive(|| "sets-load-more-btn".to_string())
                />
            </Show>
            <ImportSetPreviewModal
                is_open=preview_modal_open
                set_ids=Signal::derive(move || preview_set_ids.get())
                set_titles=Signal::derive(move || preview_set_titles.get())
                on_import_result=Callback::new(move |imported_ids: Vec<String>| {
                    if !imported_ids.is_empty() {
                        sets.update(|list| {
                            for set in list.iter_mut() {
                                if imported_ids.contains(&set.set_id) && !set.is_imported {
                                    set.is_imported = true;
                                }
                            }
                        });
                    }
                    selected_sets.set(HashSet::new());
                })
            />
        </div>
    }
}
