use dioxus::prelude::*;

use super::{Button, ButtonVariant, Paragraph};

/// EmptyState - компонент для отображения пустых состояний с призывом к действию.
/// Используется когда список элементов пуст и нужно направить пользователя.
///
/// ## Примеры использования:
/// - Пустой список карточек (EmptyCardsState)
/// - Отсутствие результатов поиска
/// - Пустая корзина покупок
/// - Нет доступных элементов в категории
///
/// ## Особенности:
/// - Настраиваемый иконка/эмодзи
/// - Заголовок и описание
/// - Кнопка действия
/// - Дополнительный контент (советы, подсказки)
#[component]
pub fn EmptyState(
    icon: Option<String>,
    title: String,
    description: Option<String>,
    action_text: Option<String>,
    on_action: Option<EventHandler<()>>,
    additional_content: Option<Element>,
) -> Element {
    rsx! {
        div { class: "text-center space-y-6 py-12",
            if let Some(icon_text) = icon {
                div { class: "text-6xl mb-6", "{icon_text}" }
            }

            Paragraph { class: Some("text-xl font-bold text-slate-700".to_string()), "{title}" }

            if let Some(desc) = description {
                Paragraph { class: Some("text-base text-slate-500 max-w-md mx-auto".to_string()),
                    "{desc}"
                }
            }

            if let Some(action) = action_text {
                if let Some(handler) = on_action {
                    div { class: "space-y-3",
                        Button {
                            variant: ButtonVariant::Rainbow,
                            class: Some("px-8 py-3 text-lg font-semibold".to_string()),
                            onclick: move |_| handler.call(()),
                            "{action}"
                        }
                        if let Some(extra) = additional_content {
                            {extra}
                        }
                    }
                }
            }
        }
    }
}
