use crate::i18n::*;
use crate::pages::lesson::kanji_card_details::KanjiCardDetails;
use crate::pages::lesson::rating_buttons_view::RatingButtonsView;
use crate::pages::shared::format_answer_text;
use crate::ui_components::{
    Card, DisplayText, Heading, HeadingLevel, KanjiDrawingPractice, Tag, TagVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, Rating};
use std::collections::HashSet;
use tracing::warn;

struct KanjiData {
    symbol: String,
    description: String,
    on_readings: Option<Vec<crate::ui_components::ReadingWithFreq>>,
    kun_readings: Option<Vec<crate::ui_components::ReadingWithFreq>>,
    radicals: Option<Vec<crate::pages::lesson::kanji_card_details::RadicalDisplay>>,
    examples: Option<Vec<(String, String)>>,
}

fn extract_kanji_data(kanji_card: &DomainCard, native_language: NativeLanguage) -> KanjiData {
    let DomainCard::Kanji(kanji) = kanji_card else {
        unreachable!()
    };

    let symbol = kanji.kanji().text().to_string();
    let description = match kanji.description(&native_language) {
        Ok(_) => format_answer_text(kanji_card, &native_language),
        Err(e) => {
            warn!(
                kanji = %symbol,
                error = %e,
                "Failed to get kanji description"
            );
            String::new()
        },
    };

    let on_readings: Option<Vec<crate::ui_components::ReadingWithFreq>> = {
        let readings = kanji.on_readings_with_freq();
        if readings.is_empty() {
            None
        } else {
            Some(readings)
        }
    };

    let kun_readings: Option<Vec<crate::ui_components::ReadingWithFreq>> = {
        let readings = kanji.kun_readings_with_freq();
        if readings.is_empty() {
            None
        } else {
            Some(readings)
        }
    };

    let radicals: Option<Vec<crate::pages::lesson::kanji_card_details::RadicalDisplay>> =
        match kanji.radicals_info() {
            Ok(r) => Some(
                r.iter()
                    .map(
                        |info| crate::pages::lesson::kanji_card_details::RadicalDisplay {
                            symbol: info.radical(),
                            name: info.name().to_string(),
                            description: info.description().to_string(),
                        },
                    )
                    .collect(),
            ),
            Err(e) => {
                warn!("Failed to get radicals for kanji: {:?}", e);
                None
            },
        };

    let examples: Option<Vec<(String, String)>> = {
        let ex: Vec<_> = kanji
            .example_words(&native_language)
            .iter()
            .map(|e| (e.word().to_string(), e.meaning().to_string()))
            .collect();
        if ex.is_empty() { None } else { Some(ex) }
    };

    KanjiData {
        symbol,
        description,
        on_readings,
        kun_readings,
        radicals,
        examples,
    }
}

fn get_card_type(i18n: &I18nContext<Locale>) -> (String, TagVariant) {
    let label = i18n
        .get_keys_untracked()
        .lesson()
        .kanji()
        .inner()
        .to_string();
    (label, TagVariant::Olive)
}

#[component]
pub fn WritingCard(
    card: DomainCard,
    on_rate: Callback<Rating>,
    #[prop(optional)] on_show_answer: Option<Callback<()>>,
    #[prop(into)] disabled: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let (card_type_label, tag_variant) = get_card_type(&i18n);

    let (symbol_char, display_text, on_readings, kun_readings, radicals, example_words) =
        match &card {
            DomainCard::Kanji(_) => {
                let data = extract_kanji_data(&card, native_language);
                (
                    data.symbol,
                    data.description,
                    data.on_readings,
                    data.kun_readings,
                    data.radicals,
                    data.examples,
                )
            },
            _ => {
                return view! { <div>{t!(i18n, lesson.writing_card_kanji_only)}</div> }.into_any();
            },
        };

    let show_details = RwSignal::new(false);

    let symbol_sv = StoredValue::new(symbol_char);
    let display_text_sv = StoredValue::new(display_text);
    let on_readings_sv = StoredValue::new(on_readings);
    let kun_readings_sv = StoredValue::new(kun_readings);
    let radicals_sv = StoredValue::new(radicals);
    let examples_sv = StoredValue::new(example_words);
    let on_rate_sv = StoredValue::new(on_rate);
    let disabled_sv = StoredValue::new(disabled);

    view! {
        <Card class=super::LESSON_CARD_CLASS shadow=true test_id="lesson-card-root">
            <div class="flex items-center justify-between mb-4">
                <Tag variant=tag_variant>
                    {card_type_label}
                </Tag>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-4">
                    <DisplayText>{display_text_sv.get_value()}</DisplayText>
                </div>

                <Show when=move || !show_details.get()>
                    <div class="my-4">
                        <KanjiDrawingPractice
                            kanji=symbol_sv.get_value()
                            on_complete=Callback::new(move |_| {
                                show_details.set(true);
                                if let Some(cb) = on_show_answer {
                                    cb.run(());
                                }
                            })
                        />
                    </div>
                </Show>

                <Show when=move || show_details.get()>
                    <Heading level=HeadingLevel::H1 class="text-6xl mb-2 text-primary text-center">
                        {symbol_sv.get_value()}
                    </Heading>

                    <KanjiCardDetails
                        kanji=symbol_sv.get_value()
                        name=display_text_sv.get_value()
                        radicals=radicals_sv.get_value()
                        example_words=examples_sv.get_value()
                        on_readings=on_readings_sv.get_value()
                        kun_readings=kun_readings_sv.get_value()
                        known_kanji=known_kanji
                        native_language=native_language
                    />

                    <div class="flex justify-center mt-4">
                        <button
                            type="button"
                            class="btn btn-ghost text-xs"
                            on:click=move |_: leptos::ev::MouseEvent| {
                                show_details.set(false);
                            }
                        >
                            {t!(i18n, lesson.writing_try_again)}
                        </button>
                    </div>

                    <RatingButtonsView on_rate=on_rate_sv.get_value() disabled=disabled_sv.get_value() />
                </Show>
            </div>
        </Card>
    }
    .into_any()
}
