use dioxus::prelude::*;

use super::H2;

/// InfoSection - компонент для отображения информационных секций с цветным фоном и заголовком.
/// Используется для группировки связанной информации в визуально выделенные блоки.
///
/// ## Цветовые темы:
/// - Neutral: Серый фон для общих секций
/// - Blue: Синий фон для информационных секций
/// - Purple: Фиолетовый фон для технических деталей
/// - Yellow: Желтый фон для предупреждений или связанных данных
/// - Green: Зеленый фон для успешных состояний

#[derive(PartialEq, Clone)]
pub enum InfoSectionTone {
    Neutral,
    Blue,
    Purple,
    Yellow,
    Green,
}

impl InfoSectionTone {
    pub fn classes(&self) -> (&'static str, &'static str) {
        match self {
            InfoSectionTone::Neutral => ("bg-slate-50", "text-slate-800"),
            InfoSectionTone::Blue => ("bg-blue-50", "text-blue-800"),
            InfoSectionTone::Purple => ("bg-purple-50", "text-purple-800"),
            InfoSectionTone::Yellow => ("bg-yellow-50", "text-yellow-800"),
            InfoSectionTone::Green => ("bg-green-50", "text-green-800"),
        }
    }
}

#[component]
pub fn InfoSection(
    title: String,
    tone: Option<InfoSectionTone>,
    class: Option<String>,
    children: Element,
) -> Element {
    let tone = tone.unwrap_or(InfoSectionTone::Neutral);
    let (bg_class, title_class) = tone.classes();
    let class_str = class.unwrap_or_default();

    rsx! {
        div { class: "{bg_class} rounded-lg p-4 space-y-3 {class_str}",
            H2 { class: Some(format!("text-lg font-semibold {}", title_class)), "{title}" }
            {children}
        }
    }
}
