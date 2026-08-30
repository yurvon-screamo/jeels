//! Построение слайдов руки знакомства из состояния юзера.
//! Переиспользуется начальной загрузкой урока и заменой карты при
//! «Уже знаю» (слайд новой карты строится на лету).

use origa::domain::Card;
use ulid::Ulid;

use super::acquaintance_state::AcquaintanceSlideData;

pub(crate) fn build_acquaintance_slides(
    user: &origa::domain::User,
    order: &[Ulid],
    native_language: origa::domain::NativeLanguage,
    i18n: &crate::i18n::I18nContext<crate::i18n::Locale>,
) -> Vec<AcquaintanceSlideData> {
    use crate::ui_components::ReadingItem;

    let answer_text = |answer: origa::domain::CardAnswer| -> String {
        match answer {
            origa::domain::CardAnswer::Text(text) => text,
            origa::domain::CardAnswer::Vocabulary {
                mut translations,
                description,
            } => {
                if let Some(description) = description {
                    translations.push(description);
                }
                translations.join(", ")
            },
        }
    };

    order
        .iter()
        .filter_map(|card_id| {
            let study_card = user.knowledge_set().get_card(*card_id)?;
            match study_card.card() {
                Card::Vocabulary(vocab) => {
                    let translations = origa::dictionary::vocabulary::get_translations(
                        vocab.word().text(),
                        &native_language,
                    )
                    .unwrap_or_default();
                    Some(AcquaintanceSlideData::Vocabulary {
                        card_id: *card_id,
                        word: vocab.word().text().to_string(),
                        pos_label: vocab
                            .pos()
                            .map(|pos| super::pos_label::part_of_speech_label(pos, i18n)),
                        translations,
                    })
                },
                Card::Kanji(kanji) => {
                    let name = kanji
                        .description(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default();
                    let radicals = kanji.radicals_info().ok().map(|infos| {
                        infos
                            .iter()
                            .map(|info| super::kanji_card_details::RadicalDisplay {
                                symbol: info.radical(),
                                name: info.name(&native_language).to_string(),
                                description: info.description(&native_language).to_string(),
                            })
                            .collect()
                    });
                    let example_words = {
                        let examples: Vec<_> = kanji
                            .example_words(&native_language)
                            .iter()
                            .map(|entry| (entry.word().to_string(), entry.meaning().to_string()))
                            .collect();
                        (!examples.is_empty()).then_some(examples)
                    };
                    let on_readings = {
                        let readings: Vec<ReadingItem> = kanji
                            .on_readings_with_freq()
                            .into_iter()
                            .map(|(reading, freq, is_rare)| ReadingItem {
                                reading,
                                freq,
                                is_rare,
                            })
                            .collect();
                        (!readings.is_empty()).then_some(readings)
                    };
                    let kun_readings = {
                        let readings: Vec<ReadingItem> = kanji
                            .kun_readings_with_freq()
                            .into_iter()
                            .map(|(reading, freq, is_rare)| ReadingItem {
                                reading,
                                freq,
                                is_rare,
                            })
                            .collect();
                        (!readings.is_empty()).then_some(readings)
                    };
                    Some(AcquaintanceSlideData::Kanji {
                        card_id: *card_id,
                        kanji: kanji.kanji().text().to_string(),
                        name,
                        radicals,
                        example_words,
                        on_readings,
                        kun_readings,
                    })
                },
                Card::Grammar(rule) => Some(AcquaintanceSlideData::Grammar {
                    card_id: *card_id,
                    title: rule
                        .title(&native_language)
                        .map(|question| question.text().to_string())
                        .unwrap_or_default(),
                    short_description: rule
                        .short_description(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default(),
                    how_to_form: rule
                        .how_to_form(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default(),
                    examples: rule
                        .examples(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default(),
                    explanation: rule
                        .explanation(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default(),
                    nuances: rule
                        .nuances(&native_language)
                        .map(&answer_text)
                        .unwrap_or_default(),
                }),
                Card::Phrase(_) => None,
            }
        })
        .collect()
}
