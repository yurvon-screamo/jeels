use dioxus::prelude::*;

use super::Card;

/// ProgressBar - универсальный компонент для отображения прогресса выполнения задач.
/// Поддерживает различные визуальные стили и состояния.
#[component]
pub fn ProgressBar(
    /// Текущий прогресс в процентах (0.0 - 100.0)
    progress: f64,
    /// Заголовок прогресс бара (опционально)
    title: Option<String>,
    /// Подзаголовок или дополнительная информация (опционально)
    subtitle: Option<String>,
    /// Дополнительный CSS класс
    class: Option<String>,
    /// Показывать ли процент в виде текста
    show_percentage: Option<bool>,
    /// Размер прогресс бара (sm, md, lg)
    size: Option<String>,
) -> Element {
    let percentage = (progress as usize).min(100);
    let show_percentage = show_percentage.unwrap_or(true);
    let size = size.unwrap_or("md".to_string());
    let class_str = class.unwrap_or_default();

    let (bar_height, text_size) = match size.as_str() {
        "sm" => ("h-1.5", "text-xs"),
        "lg" => ("h-4", "text-base"),
        _ => ("h-2", "text-sm"), // md
    };

    rsx! {
        Card { class: Some(format!("py-3 px-4 {}", class_str)),
            if let Some(title_text) = title {
                div { class: "flex items-center justify-between {text_size}",
                    div { class: "flex items-center gap-2",
                        span { class: "font-semibold text-slate-700", "{title_text}" }
                        if show_percentage {
                            span { class: "text-xs bg-slate-100 text-slate-600 px-2 py-0.5 rounded-full",
                                "{percentage}%"
                            }
                        }
                    }
                    if let Some(subtitle_text) = subtitle {
                        span { class: "text-slate-500 font-medium", "{subtitle_text}" }
                    }
                }
            }

            div { class: "w-full {bar_height} bg-slate-100 rounded-full overflow-hidden mt-2",
                div {
                    class: format!(
                        "{bar_height} rounded-full transition-all duration-700 ease-out relative {}",
                        if progress >= 100.0 {
                            "bg-gradient-to-r from-green-400 to-green-500"
                        } else {
                            "bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500"
                        },
                    ),
                    style: "width: {progress}%",

                    if progress > 0.0 && progress < 100.0 {
                        div { class: "absolute inset-0 bg-gradient-to-r from-transparent via-white to-transparent animate-pulse opacity-30" }
                    }
                }
            }
        }
    }
}
