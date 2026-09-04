use std::collections::HashSet;

use leptos::prelude::*;
use origa::dictionary::grammar::{CommonMistake, NuanceNote};

use crate::ui_components::FuriganaText;

/// Structured render of grammar rule nuances (schema v2): wrong/correct
/// mistake pairs plus free-form notes. The data carries no emoji or other
/// presentation markers — visual grammar lives here.
#[component]
pub fn NuancesSection(
    common_mistakes: Vec<CommonMistake>,
    notes: Vec<NuanceNote>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);
    let common_mistakes_stored = StoredValue::new(common_mistakes);
    let notes_stored = StoredValue::new(notes);

    view! {
        <div class="grammar-nuances" data-testid=move || test_id.get()>
            <For
                each=move || common_mistakes_stored.get_value().into_iter().enumerate()
                key=|(index, _): &(usize, CommonMistake)| *index
                children=move |(_index, mistake)| {
                    let note = StoredValue::new(
                        mistake.note().map(|n| n.to_string()).unwrap_or_default(),
                    );
                    view! {
                        <div class="grammar-nuance-mistake">
                            <div class="grammar-nuance-wrong">
                                <FuriganaText
                                    text=mistake.wrong().to_string()
                                    known_kanji=known_kanji_stored.get_value()
                                />
                            </div>
                            <span class="grammar-nuance-arrow" aria-hidden="true">"→"</span>
                            <div class="grammar-nuance-correct">
                                <FuriganaText
                                    text=mistake.correct().to_string()
                                    known_kanji=known_kanji_stored.get_value()
                                />
                            </div>
                        </div>
                        <Show when=move || !note.get_value().is_empty()>
                            <div class="grammar-nuance-mistake-note">
                                <FuriganaText
                                    text=note.get_value()
                                    known_kanji=known_kanji_stored.get_value()
                                />
                            </div>
                        </Show>
                    }
                }
            />
            <Show when=move || !notes_stored.get_value().is_empty()>
                <ul class="grammar-nuance-notes">
                    <For
                        each=move || notes_stored.get_value().into_iter().enumerate()
                        key=|(index, _): &(usize, NuanceNote)| *index
                        children=move |(_index, note)| {
                            view! {
                                <li class="grammar-nuance-note-item">
                                    <FuriganaText
                                        text=note.text().to_string()
                                        known_kanji=known_kanji_stored.get_value()
                                    />
                                </li>
                            }
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}
