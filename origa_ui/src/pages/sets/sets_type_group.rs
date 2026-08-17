use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use lexical_sort::natural_lexical_cmp;
use std::collections::HashSet;

use super::filters::{TypeFilter, available_set_types};
use super::set_card::SetCard;
use super::types::SetInfo;

fn get_set_type_label(set_type_id: &str) -> String {
    available_set_types()
        .iter()
        .find(|t| t.id == set_type_id)
        .map(|t| t.label_ru.clone())
        .unwrap_or_else(|| set_type_id.to_string())
}

#[component]
pub fn SetsTypeGroup(
    set_type: String,
    sets_for_level: Memo<Vec<SetInfo>>,
    type_filter: RwSignal<TypeFilter>,
    known_kanji: HashSet<char>,
    on_import: Callback<(String, String)>,
    selected_sets: RwSignal<HashSet<String>>,
    on_toggle_select: Callback<String>,
) -> impl IntoView {
    let set_type_clone = set_type.clone();
    let sets_for_type = Memo::new(move |_| {
        let current_filter = type_filter.get();
        let mut sets: Vec<_> = sets_for_level
            .get()
            .into_iter()
            .filter(|s| s.set_type == set_type_clone && current_filter.matches(&set_type_clone))
            .collect();

        sets.sort_by(|a, b| natural_lexical_cmp(&a.title, &b.title));
        sets
    });

    let known_kanji_stored = StoredValue::new(known_kanji);
    let set_type_label = StoredValue::new(get_set_type_label(&set_type));

    view! {
        <Show when=move || !sets_for_type.get().is_empty()>
            <div class="mb-4">
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    class="mb-2"
                >
                    {move || set_type_label.get_value()}
                </Text>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For
                        each=move || sets_for_type.get()
                        key=|s| format!("{}-{}", s.set_id, s.is_imported)
                        children=move |set_info| {
                            view! {
                                <SetCard
                                    set_info=set_info
                                    known_kanji=known_kanji_stored.get_value()
                                    on_import=on_import
                                    selected_sets=selected_sets
                                    on_toggle_select=on_toggle_select
                                />
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}
