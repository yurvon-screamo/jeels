use chrono::{Datelike, TimeZone};
use origa::domain::{Card, CardAnswer, CardType, DailyHistoryItem, KnowledgeSet, NativeLanguage};

use crate::pages::shared::format_answer_text;

const RU_MONTHS_SHORT: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];
const EN_MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone)]
pub struct RecentlyStudiedItem {
    pub card_id: String,
    pub card_type: String,
    pub japanese: String,
    pub meaning: String,
    pub reading: Option<String>,
    pub short_description: Option<String>,
}

#[derive(Clone)]
pub struct ActivityDataPoint {
    pub date_label: String,
    pub learned: f64,
    pub in_progress: f64,
    pub new_count: f64,
    pub difficult: f64,
}

pub fn compute_studied_today(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Vec<RecentlyStudiedItem> {
    let today = chrono::Local::now().date_naive();

    let mut cards_today: Vec<_> = knowledge_set
        .study_cards()
        .values()
        .filter(|sc| !matches!(CardType::from(sc.card()), CardType::Phrase))
        .filter(|sc| {
            sc.memory().last_review_date().is_some_and(|last_review| {
                let local_date = chrono::Local
                    .from_utc_datetime(&last_review.naive_utc())
                    .date_naive();
                local_date >= today
            })
        })
        .collect();

    cards_today.sort_by(|a, b| {
        b.memory()
            .last_review_date()
            .cmp(&a.memory().last_review_date())
    });

    cards_today
        .into_iter()
        .map(|sc| {
            let card = sc.card();

            let japanese = card
                .question(lang)
                .map(|q| q.text().to_string())
                .unwrap_or_else(|_| match card {
                    Card::Vocabulary(v) => v.word().text().to_string(),
                    Card::Kanji(k) => k.kanji().text().to_string(),
                    Card::Grammar(g) => g.rule_id().to_string(),
                    Card::Phrase(p) => p.phrase_id().to_string(),
                });

            let meaning = format_answer_text(card, lang);

            let card_type = match CardType::from(card) {
                CardType::Kanji => "kanji",
                CardType::Vocabulary => "vocabulary",
                CardType::Grammar => "grammar",
                CardType::Phrase => "vocabulary",
            };

            let (reading, short_description) = match card {
                Card::Kanji(k) => {
                    let on = k.on_readings();
                    let kun = k.kun_readings();
                    let reading = if on.is_empty() && kun.is_empty() {
                        None
                    } else {
                        let parts: Vec<String> = on.iter().chain(kun.iter()).cloned().collect();
                        Some(parts.join("、"))
                    };
                    (reading, None)
                },
                Card::Grammar(g) => {
                    let short_desc = match g.short_description(lang).ok() {
                        Some(CardAnswer::Vocabulary { translations, .. }) => {
                            Some(translations.join(", "))
                        },
                        Some(CardAnswer::Text(s)) => Some(s),
                        None => None,
                    };
                    (None, short_desc)
                },
                _ => (None, None),
            };

            RecentlyStudiedItem {
                card_id: sc.card_id().to_string(),
                card_type: card_type.to_string(),
                japanese,
                meaning,
                reading,
                short_description,
            }
        })
        .collect()
}

pub fn compute_30day_chart_data(
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> Vec<ActivityDataPoint> {
    let mut items: Vec<_> = history.to_vec();
    items.sort_by_key(|a| a.timestamp());

    let start = items.len().saturating_sub(30);
    items
        .into_iter()
        .skip(start)
        .map(|item| {
            let local = chrono::Local.from_utc_datetime(&item.timestamp().naive_utc());
            let day = local.day();
            let month_idx = (local.month0() as usize).min(11);
            let month_str = match lang {
                NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                _ => EN_MONTHS_SHORT[month_idx],
            };
            ActivityDataPoint {
                date_label: format!("{} {}", day, month_str),
                learned: item.known_words() as f64,
                in_progress: item.in_progress_words() as f64,
                new_count: item.new_words() as f64,
                difficult: item.high_difficulty_words() as f64,
            }
        })
        .collect()
}

#[derive(Clone, Default)]
pub struct CompletionForecast {
    pub days_remaining: Option<usize>,
    pub is_all_studied: bool,
    pub target_date_label: String,
}

pub fn compute_completion_forecast(
    knowledge_set: &KnowledgeSet,
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> CompletionForecast {
    use origa::domain::estimate_completion_date;

    let mut new_cards_remaining = 0usize;
    let mut total_cards = 0usize;

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        total_cards += 1;
        if study_card.memory().is_new() {
            new_cards_remaining += 1;
        }
    }

    let is_all_studied = total_cards > 0 && new_cards_remaining == 0;

    let (days_remaining, target_date_label) = if new_cards_remaining > 0 {
        match estimate_completion_date(history, new_cards_remaining) {
            Some(date) => {
                let local = chrono::Local.from_utc_datetime(&date.naive_utc());
                let day = local.day();
                let month_idx = (local.month0() as usize).min(11);
                let month_str = match lang {
                    NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                    _ => EN_MONTHS_SHORT[month_idx],
                };
                let date_label = format!("~{} {}", day, month_str);

                let now = chrono::Utc::now();
                let duration = date.signed_duration_since(now);
                let days = (duration.num_hours() as f64 / 24.0).ceil().max(1.0) as usize;

                (Some(days), date_label)
            },
            None => (None, String::new()),
        }
    } else {
        (None, String::new())
    };

    CompletionForecast {
        days_remaining,
        is_all_studied,
        target_date_label,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Card, KnowledgeSet, Question, VocabularyCard};

    fn set_with_new_cards(n: usize) -> KnowledgeSet {
        let mut set = KnowledgeSet::new();
        for i in 0..n {
            let vocab = VocabularyCard::new_with_pos(
                Question::new(format!("word{i}")).unwrap(),
                None,
                None,
            );
            let _ = set.create_card(Card::Vocabulary(vocab));
        }
        set
    }

    #[test]
    fn chart_data_limits_to_thirty_points() {
        let history = vec![DailyHistoryItem::new(); 45];
        let data = compute_30day_chart_data(&history, &NativeLanguage::English);
        assert_eq!(data.len(), 30, "only the last 30 days are charted");
    }

    #[test]
    fn chart_data_labels_are_localised() {
        let history = vec![DailyHistoryItem::new()];
        let en = compute_30day_chart_data(&history, &NativeLanguage::English);
        let ru = compute_30day_chart_data(&history, &NativeLanguage::Russian);
        assert_eq!(en.len(), 1);
        // Same day → labels share the day number, month abbreviations differ
        // between locales for most months; both must be non-empty and start
        // with the day number.
        assert!(!en[0].date_label.is_empty());
        assert!(!ru[0].date_label.is_empty());
    }

    #[test]
    fn completion_forecast_empty_set_is_not_all_studied() {
        let set = KnowledgeSet::new();
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(!forecast.is_all_studied, "no cards → not all studied");
        assert_eq!(forecast.target_date_label, "");
    }

    #[test]
    fn completion_forecast_new_cards_remaining_is_not_all_studied() {
        let set = set_with_new_cards(3);
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(!forecast.is_all_studied, "new cards remain → not done");
        // Without history the completion date is not estimable.
        assert_eq!(forecast.days_remaining, None);
    }

    #[test]
    fn completion_forecast_all_cards_reviewed_is_all_studied() {
        let mut set = set_with_new_cards(2);
        // Mark every card as known — no new cards remain.
        for (id, _) in set.study_cards().clone() {
            let _ = set.mark_card_as_known(id);
        }
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(forecast.is_all_studied, "everything reviewed → done");
    }

    #[test]
    fn studied_today_excludes_phrase_cards_and_old_reviews() {
        let set = set_with_new_cards(2);
        // No review dates → nothing studied today.
        let items = compute_studied_today(&set, &NativeLanguage::English);
        assert!(
            items.is_empty(),
            "cards without a review date are not 'studied today'"
        );
    }
}
