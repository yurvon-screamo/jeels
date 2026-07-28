use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::dictionary::kanji::{KanjiInfo, get_kanji_list, sort_by_difficulty};
use origa::domain::{Card, JapaneseLevel};
use origa::traits::UserRepository;
use std::collections::HashSet;

#[derive(Clone)]
pub struct ModalState {
    pub selected_level: RwSignal<JapaneseLevel>,
    pub available_kanji: RwSignal<Vec<&'static KanjiInfo>>,
    pub selected_kanji: RwSignal<HashSet<String>>,
    pub is_loading_kanji: RwSignal<bool>,
    pub is_creating: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub repository: HybridUserRepository,
    pub refresh_trigger: RwSignal<u32>,
    pub search_query: RwSignal<String>,
}

impl ModalState {
    pub fn new(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> Self {
        let repository =
            use_context::<HybridUserRepository>().expect("repository context not provided");

        let selected_kanji = RwSignal::new(HashSet::new());

        Effect::new({
            let selected_kanji_clone = selected_kanji;
            move |_| {
                if is_open.get() {
                    selected_kanji_clone.set(HashSet::new());
                }
            }
        });

        Self {
            selected_level: RwSignal::new(JapaneseLevel::N5),
            available_kanji: RwSignal::new(Vec::new()),
            selected_kanji,
            is_loading_kanji: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            error_message: RwSignal::new(None),
            repository,
            refresh_trigger,
            search_query: RwSignal::new(String::new()),
        }
    }

    pub fn load_kanji(&self) {
        let level = self.selected_level.get();
        let repository = self.repository.clone();
        let available_kanji = self.available_kanji;
        let is_loading = self.is_loading_kanji;
        let error = self.error_message;
        let disposed = StoredValue::new(());

        is_loading.set(true);
        error.set(None);

        spawn_local(async move {
            match repository.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    let learned_kanji: HashSet<String> = user
                        .knowledge_set()
                        .study_cards()
                        .values()
                        .filter_map(|card| {
                            if let Card::Kanji(kanji_card) = card.card() {
                                Some(kanji_card.kanji().text().to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    let kanji_list: Vec<&'static KanjiInfo> = {
                        let mut list: Vec<&'static KanjiInfo> = get_kanji_list(&level)
                            .into_iter()
                            .filter(|kanji_info| {
                                !learned_kanji.contains(&kanji_info.kanji().to_string())
                            })
                            .collect();
                        sort_by_difficulty(&mut list);
                        list
                    };

                    available_kanji.set(kanji_list);
                    is_loading.set(false);
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(
                        crate::i18n::use_i18n()
                            .get_keys_untracked()
                            .shared()
                            .user_not_found()
                            .inner()
                            .to_string(),
                    ));
                    is_loading.set(false);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    error.set(Some(e.to_string()));
                    is_loading.set(false);
                },
            }
        });
    }

    pub fn select_level(&self, level: JapaneseLevel) {
        self.selected_level.set(level);
        self.selected_kanji.set(HashSet::new());
        self.search_query.set(String::new());
        self.load_kanji();
    }

    pub fn reset(&self) {
        self.error_message.set(None);
        self.selected_kanji.set(HashSet::new());
    }

    pub fn select_all(&self) {
        let all_kanji: HashSet<String> = self
            .available_kanji
            .get()
            .iter()
            .map(|k| k.kanji().to_string())
            .collect();
        self.selected_kanji.set(all_kanji);
    }
}
