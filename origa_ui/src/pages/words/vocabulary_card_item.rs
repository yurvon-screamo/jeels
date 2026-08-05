use std::collections::HashSet;

use super::super::shared::{CardStatus, DeleteRequest, format_answer_parts};
use crate::i18n::use_i18n;
use crate::ui_components::{
    CardActionBar, DeleteConfirmModal, FsrsMetrics, FuriganaText, Tag, TagVariant, WordTranslations,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

#[component]
pub fn VocabularyCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    known_kanji: HashSet<char>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<()>,
    on_delete: Callback<DeleteRequest>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_id = *study_card.card_id();
    let is_favorite = study_card.is_favorite();
    let memory = study_card.memory();

    let is_delete_modal_open = RwSignal::new(false);

    let confirm_delete = Callback::new(move |_| {
        on_delete.run(DeleteRequest {
            card_id,
            on_success: Callback::new(move |_| is_delete_modal_open.set(false)),
        })
    });

    let word = match study_card.card() {
        DomainCard::Vocabulary(vocab) => vocab.word().text().to_string(),
        _ => "?".to_string(),
    };

    let study_card_for_meaning = study_card.clone();
    let answer_data = Memo::new(move |_| {
        let lang = native_language.get();
        match study_card_for_meaning.card() {
            DomainCard::Vocabulary(_) => {
                let (translations, description) =
                    format_answer_parts(study_card_for_meaning.card(), &lang);
                if translations.is_empty() {
                    (vec!["?".to_string()], None)
                } else {
                    (translations, description)
                }
            },
            _ => (vec!["?".to_string()], None),
        }
    });

    let translations = Signal::derive(move || answer_data.get().0);
    let description = Signal::derive(move || answer_data.get().1);

    let status = CardStatus::from_study_card(&study_card);
    let show_mark_as_known = status != CardStatus::Learned;

    let known_kanji_for_furigana = known_kanji;

    view! {
        <div class="word-card anima-lift" data-testid="words-card-item">
            <div class="word-card-badge">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {move || status.label(&i18n)}
                </Tag>
            </div>
            <div class="word-card-body">
                <div class="word-card-word-box">
                    <FuriganaText text=word known_kanji=known_kanji_for_furigana/>
                </div>
                <div class="word-card-content">
                    <WordTranslations
                        translations=translations
                        description=description
                        test_id=Signal::derive(|| "words-card-translations".to_string())
                    />
                </div>
            </div>
            <div class="word-card-divider"></div>
            <div class="word-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "words-card-fsrs".to_string())
                />
                <div class="word-card-actions">
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        show_mark_as_known=Signal::derive(move || show_mark_as_known)
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(()))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "words-card-item".to_string())
                        show_tag=Signal::derive(|| false)
                    />
                </div>
            </div>
        </div>
        <DeleteConfirmModal
            test_id="words-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
