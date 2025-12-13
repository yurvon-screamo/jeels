use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::select_cards_to_learn::SelectCardsToLearnUseCase;
use keikaku::domain::study_session::StudySessionItem;

#[derive(Clone, PartialEq)]
pub enum SessionState {
    Settings,
    Loading,
    Active,
    Completed,
}

#[derive(Clone, PartialEq)]
pub enum LearnStep {
    Question,
    Answer,
}

#[derive(Clone, PartialEq)]
pub struct LearnCard {
    pub id: String,
    pub question: String,
    pub answer: String,
}

pub fn use_learn_session() -> UseLearnSession {
    use_hook(|| UseLearnSession {
        state: use_signal(|| SessionState::Settings),
        cards: use_signal(Vec::new),
        current_index: use_signal(|| 0),
        current_step: use_signal(|| LearnStep::Question),
        limit: use_signal(|| "7".to_string()),
        show_furigana: use_signal(|| false),
        loading: use_signal(|| false),
    })
}

#[derive(Clone, PartialEq)]
pub struct UseLearnSession {
    pub state: Signal<SessionState>,
    pub cards: Signal<Vec<LearnCard>>,
    pub current_index: Signal<usize>,
    pub current_step: Signal<LearnStep>,
    pub limit: Signal<String>,
    pub show_furigana: Signal<bool>,
    pub loading: Signal<bool>,
}

impl UseLearnSession {
    pub fn current_card(&self) -> Option<LearnCard> {
        let cards = (self.cards)();
        let index = (self.current_index)();
        cards.get(index).cloned()
    }

    pub fn progress(&self) -> f64 {
        let cards = (self.cards)();
        if cards.is_empty() {
            0.0
        } else {
            let index = (self.current_index)();
            (index as f64 + 1.0) / cards.len() as f64 * 100.0
        }
    }

    pub fn has_cards(&self) -> bool {
        !(self.cards)().is_empty()
    }

    pub fn is_completed(&self) -> bool {
        let index = (self.current_index)();
        let cards_len = (self.cards)().len();
        index >= cards_len.saturating_sub(1)
    }

    pub fn start_session(&self) {
        let limit_str = (self.limit)();
        let mut session = self.clone();

        spawn(async move {
            session.loading.set(true);
            session.state.set(SessionState::Loading);

            let limit_val = limit_str.parse::<usize>().ok();
            match session.fetch_cards(limit_val).await {
                Ok(new_cards) => {
                    if new_cards.is_empty() {
                        session.state.set(SessionState::Settings);
                        session.cards.set(vec![]);
                    } else {
                        session.cards.set(new_cards);
                        session.current_index.set(0);
                        session.current_step.set(LearnStep::Question);
                        session.state.set(SessionState::Active);
                    }
                    session.loading.set(false);
                }
                Err(e) => {
                    session.cards.set(vec![]);
                    session.state.set(SessionState::Settings);
                    session.current_step.set(LearnStep::Question);
                    session.loading.set(false);
                    error!("learn fetch error: {}", e);
                }
            }
        });
    }

    pub fn next_card(&mut self) {
        let current_index = (self.current_index)();
        let cards_len = (self.cards)().len();
        if current_index + 1 < cards_len {
            self.current_index.set(current_index + 1);
            self.current_step.set(LearnStep::Question);
        } else {
            self.state.set(SessionState::Completed);
        }
    }

    pub fn reset_session(&mut self) {
        self.cards.set(vec![]);
        self.current_index.set(0);
        self.current_step.set(LearnStep::Question);
        self.state.set(SessionState::Settings);
    }

    pub async fn fetch_cards(&self, limit: Option<usize>) -> Result<Vec<LearnCard>, String> {
        let env = init_env().await?;
        let repo = env.get_repository().await.map_err(to_error)?;
        let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
        let items = SelectCardsToLearnUseCase::new(repo)
            .execute(user_id, false, false, limit)
            .await
            .map_err(to_error)?;
        Ok(items.into_iter().filter_map(map_item).collect::<Vec<_>>())
    }
}

fn map_item(item: StudySessionItem) -> Option<LearnCard> {
    match item {
        StudySessionItem::Vocabulary(v) => Some(LearnCard {
            id: v.card_id().to_string(),
            question: v.word().to_string(),
            answer: v.meaning().to_string(),
        }),
        StudySessionItem::Kanji(k) => Some(LearnCard {
            id: k.card_id().to_string(),
            question: k.kanji().to_string(),
            answer: k.description().to_string(),
        }),
    }
}

#[component]
pub fn QuestionView(
    question: String,
    show_furigana: bool,
    on_show_answer: EventHandler<MouseEvent>,
) -> Element {
    use crate::domain::WordCard;
    use crate::ui::{Button, ButtonVariant, Paragraph};

    rsx! {
        div { class: "space-y-4",
            WordCard { text: question, show_furigana }
            div { class: "space-y-2",
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: on_show_answer,
                    "Показать ответ (Пробел)"
                }
                Paragraph { class: Some("text-xs text-center text-slate-400".to_string()),
                    "Нажмите Пробел или кнопку выше"
                }
            }
        }
    }
}

#[component]
pub fn AnswerView(
    question: String,
    answer: String,
    show_furigana: bool,
    session: UseLearnSession,
) -> Element {
    rsx! {
        div { class: "space-y-4",
            crate::domain::CardAnswer {
                question,
                answer,
                show_furigana,
                examples: None,
            }
            RatingSection { on_rate: // TODO: Implement rating
                move |rating| {} }
        }
    }
}

#[component]
pub fn RatingSection(on_rate: EventHandler<u8>) -> Element {
    use crate::domain::RatingButtons;

    rsx! {
        RatingButtons {
            on_rate: move |rating| {
                // Преобразовать Rating в u8
                let rating_value = match rating {
                    crate::domain::Rating::Easy => 1,
                    crate::domain::Rating::Good => 2,
                    crate::domain::Rating::Hard => 3,
                    crate::domain::Rating::Again => 4,
                };
                on_rate.call(rating_value);
            },
        }
    }
}

#[component]
pub fn QuestionCard(question: String, show_furigana: bool) -> Element {
    rsx! {
        crate::domain::WordCard { text: question, show_furigana }
    }
}

#[component]
pub fn AnswerCard(question: String, answer: String, show_furigana: bool) -> Element {
    rsx! {
        crate::domain::CardAnswer {
            question,
            answer,
            show_furigana,
            examples: None,
        }
    }
}

#[component]
pub fn RatingButton(
    rating: u8,
    label: String,
    color: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    use crate::ui::Button;

    rsx! {
        button {
            class: "{color} text-white font-bold py-4 px-4 rounded-xl shadow-md hover:shadow-lg active:scale-95 transition-all duration-200 text-sm",
            onclick: move |e| onclick.call(e),
            div { class: "space-y-1",
                span { class: "block text-xs opacity-90", "Клавиша {rating}" }
                span { class: "block text-base", {label} }
            }
        }
    }
}

pub fn handle_key_action(action: crate::components::KeyAction) {
    // TODO: Implement key handling
}
