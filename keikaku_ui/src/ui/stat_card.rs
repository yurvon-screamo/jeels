use dioxus::prelude::*;

use super::Card;

/// StatCard - универсальный компонент для отображения статистических данных.
///
/// ## Простой режим (без title):
/// - Центрированное отображение числа с подписью
/// - Используется для результатов, счетчиков, простых метрик
/// - Пример: StatCard { value: "42", label: "пройдено" }
///
/// ## Расширенный режим (с title):
/// - Заголовок, число и цветовая индикация в карточке
/// - Используется для статистики карточек, дашбордов, детальных метрик
/// - Пример: StatCard { title: Some("Всего"), value: "150", label: "", tone: Some(MetricTone::Neutral) }
///
/// ## Цветовые темы:
/// - Neutral: Стандартный серый цвет (text-slate-800)
/// - Warning: Оранжевый для предупреждений (text-amber-600)
/// - Success: Зеленый для успеха (text-green-600)

#[derive(PartialEq, Clone)]
pub enum MetricTone {
    Neutral,
    Warning,
    Success,
}

impl MetricTone {
    pub fn text_classes(&self) -> &'static str {
        match self {
            MetricTone::Neutral => "text-slate-800",
            MetricTone::Warning => "text-amber-600",
            MetricTone::Success => "text-green-600",
        }
    }
}

#[component]
pub fn StatCard(
    value: String,
    label: String,
    title: Option<String>,
    tone: Option<MetricTone>,
    class: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();

    if let Some(card_title) = title {
        let tone = tone.unwrap_or(MetricTone::Neutral);
        let base_classes = "p-4";

        rsx! {
            Card { class: Some(format!("{} {}", base_classes, class_str)),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "{card_title}"
                        }
                        div { class: format!("text-2xl font-bold mt-1 {}", tone.text_classes()),
                            "{value}"
                        }
                        div { class: "text-xs text-slate-400 mt-1", "{label}" }
                    }
                }
            }
        }
    } else {
        // Simple mod
        rsx! {
            div { class: "text-center {class_str}",
                div { class: "text-3xl font-bold text-blue-600", "{value}" }
                div { class: "text-sm text-slate-500", "{label}" }
            }
        }
    }
}
