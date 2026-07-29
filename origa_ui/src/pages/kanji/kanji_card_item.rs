use super::super::shared::{CardStatus, DeleteRequest, format_answer_text};
use crate::i18n::use_i18n;
use crate::ui_components::{CardActionBar, DeleteConfirmModal, FsrsMetrics, Tag, TagVariant};
use leptos::prelude::*;
use leptos_router::components::A;
use origa::domain::{Card as DomainCard, NativeLanguage, StudyCard};
use ulid::Ulid;

const RADICALS_MAX_LEN: usize = 20;

#[component]
pub fn KanjiCardItem(
    study_card: StudyCard,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    on_toggle_favorite: Callback<Ulid>,
    on_mark_as_known: Callback<Ulid>,
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

    let (kanji_char, radicals) = match study_card.card() {
        DomainCard::Kanji(kanji_card) => (
            kanji_card.kanji().text().to_string(),
            kanji_card.radicals_chars().into_iter().collect::<String>(),
        ),
        _ => ("?".to_string(), String::new()),
    };

    let study_card_for_answer = study_card.clone();
    let answer_text = Memo::new(move |_| {
        let lang = native_language.get();
        format_answer_text(study_card_for_answer.card(), &lang)
    });

    // Compact readings preview: top-N non-rare readings sorted by frequency
    // desc. Falls back to top-N of any readings if all are rare. Kept on a
    // single line with ellipsis — the goal is a glanceable preview, not a
    // reference list (rarity details live on the detail page).
    let compact_readings = match study_card.card() {
        DomainCard::Kanji(kanji_card) => {
            let mut on = kanji_card.on_readings_with_freq();
            let mut kun = kanji_card.kun_readings_with_freq();
            let mut all = std::mem::take(&mut on);
            all.append(&mut kun);
            all.sort_by_key(|(_, f)| std::cmp::Reverse(*f));
            let non_rare: Vec<(String, u32)> = all
                .iter()
                .filter(|(_, f)| *f > origa::domain::RARE_READING_MAX_FREQ)
                .cloned()
                .collect();
            let source = if non_rare.is_empty() { &all } else { &non_rare };
            source
                .iter()
                .take(3)
                .map(|(r, _)| r.clone())
                .collect::<Vec<_>>()
                .join(" · ")
        },
        _ => String::new(),
    };
    let compact_readings_stored = StoredValue::new(compact_readings);

    let status = CardStatus::from_study_card(&study_card);
    let show_mark_as_known = status != CardStatus::Learned;

    let show_radicals = !radicals.is_empty() && radicals.len() <= RADICALS_MAX_LEN;
    let radicals_text = Signal::derive({
        let radicals = radicals.clone();
        move || {
            i18n.get_keys()
                .shared()
                .radicals_label()
                .inner()
                .to_string()
                .replacen("{}", &radicals, 1)
        }
    });

    view! {
        <div class="kanji-card anima-lift" data-testid="kanji-card-item">
            <div class="kanji-card-badge">
                <Tag variant=Signal::derive(move || status.tag_variant())>
                    {move || status.label(&i18n)}
                </Tag>
            </div>
            <A href=format!("/kanji/{}", card_id) attr:class="kanji-card-link">
                <div class="kanji-card-kanji-box">
                    <span class="kanji-card-kanji-char">{kanji_char}</span>
                </div>
                <div class="kanji-card-content">
                    <Show when=move || !answer_text.get().is_empty()>
                        <span class="kanji-card-answer">{move || answer_text.get()}</span>
                    </Show>
                    <Show when=move || !compact_readings_stored.get_value().is_empty()>
                        <span
                            class="kanji-card-compact-readings"
                            data-testid="kanji-card-compact-readings"
                        >
                            {compact_readings_stored.get_value()}
                        </span>
                    </Show>
                    <Show when=move || show_radicals>
                        <span class="kanji-card-radicals">{radicals_text}</span>
                    </Show>
                </div>
            </A>
            <div class="kanji-card-divider"></div>
            <div class="kanji-card-footer">
                <FsrsMetrics
                    difficulty=memory.difficulty().map(|d| d.value())
                    stability=memory.stability().map(|s| s.value())
                    test_id=Signal::derive(|| "kanji-card-fsrs".to_string())
                />
                <div class="kanji-card-actions">
                    <CardActionBar
                        tag_variant=TagVariant::default()
                        tag_label=Signal::derive(|| "".to_string())
                        is_favorite=Signal::derive(move || is_favorite)
                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id))
                        show_mark_as_known=Signal::derive(move || show_mark_as_known)
                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id))
                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                        test_id=Signal::derive(|| "kanji-card-item".to_string())
                        show_tag=Signal::derive(|| false)
                    />
                </div>
            </div>
        </div>
        <DeleteConfirmModal
            test_id="kanji-delete-modal"
            is_open=is_delete_modal_open
            is_deleting=is_deleting
            on_confirm=confirm_delete
            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
        />
    }
}
