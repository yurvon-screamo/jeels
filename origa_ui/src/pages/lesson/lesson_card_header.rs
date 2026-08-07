use crate::i18n::use_i18n;
use crate::ui_components::{AudioButtons, Tag};
use leptos::prelude::*;
use origa::domain::{Card, GrammarInfo};

use super::card_type::CardType;
use super::grammar_info_badge::GrammarInfoBadge;
use super::pos_label::part_of_speech_label;

#[component]
pub fn LessonCardHeader(
    card_type: CardType,
    question_text: String,
    grammar_info: Option<GrammarInfo>,
    show_answer: Signal<bool>,
    card: Card,
    #[prop(into)] audio_path: Option<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let grammar_info = StoredValue::new(grammar_info);
    let pos_label = StoredValue::new(
        card.vocabulary_part_of_speech()
            .map(|p| part_of_speech_label(p, &i18n)),
    );
    view! {
        <div class="flex items-center justify-between mb-4 gap-2">
            <div class="flex items-center gap-2 flex-wrap min-w-0">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                <Show when=move || pos_label.get_value().is_some()>
                    {move || {
                        pos_label
                            .get_value()
                            .map(|label| {
                                view! {
                                    // POS is secondary metadata (a sub-classification of the
                                    // Vocabulary card), not a primary category. DESIGN.md
                                    // assigns the muted Tertiary tier to secondary metadata
                                    // and reserves coloured Tag variants for distinguishing
                                    // card TYPES — keep Default here.
                                    <Tag>
                                        {label}
                                    </Tag>
                                }
                            })
                    }}
                </Show>
                <Show when=move || show_answer.get() && grammar_info.get_value().is_some()>
                    {move || {
                        grammar_info
                            .get_value()
                            .map(|info| {
                                view! {
                                    <GrammarInfoBadge title=info.title().to_string() />
                                }
                            })
                    }}
                </Show>
            </div>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons text=question_text.clone() audio_path=audio_path.clone() class=Signal::derive(|| "shrink-0".to_string())/>
            </Show>
        </div>
    }
}
