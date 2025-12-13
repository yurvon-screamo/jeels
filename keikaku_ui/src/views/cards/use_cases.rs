use keikaku::domain::VocabularyCard;

use crate::domain::{UiCard};

pub fn map_card(card: &VocabularyCard) -> UiCard {
    let next_review = card
        .memory()
        .next_review_date()
        .map(|d| {
            let now = chrono::Utc::now();
            let diff = (*d - now).num_days();

            if diff < 0 {
                "Просрочено".to_string()
            } else if diff == 0 {
                "Сегодня".to_string()
            } else if diff == 1 {
                "Завтра".to_string()
            } else if diff < 7 {
                format!("Через {} дн.", diff)
            } else {
                d.format("%d.%m.%Y").to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string());

    UiCard {
        id: card.id().to_string(),
        question: card.word().text().to_string(),
        answer: card.meaning().text().to_string(),
        next_review,
        due: card.memory().is_due(),
    }
}
