use dioxus::prelude::*;

use super::Select;

/// LabeledSelect - компонент Select с лейблом и дополнительными опциями.
/// Обертка над базовым Select компонентом для удобства использования в формах.
///
/// ## Примеры использования:
/// - Фильтры по статусу, сортировке
/// - Выбор категории, типа, приоритета
/// - Любые селекторы с описательными лейблами
///
/// ## Особенности:
/// - Автоматическое управление состоянием выбора
/// - Поддержка различных типов опций
/// - Гибкая настройка внешнего вида
#[component]
pub fn LabeledSelect<T>(
    label: String,
    options: Vec<T>,
    selected: Option<T>,
    onselect: EventHandler<T>,
    class: Option<String>,
) -> Element
where
    T: Clone + PartialEq + std::fmt::Display + 'static,
{
    let class_str = class.unwrap_or_default();

    rsx! {
        div { class: "flex-1 min-w-[200px] {class_str}",
            Select {
                label: Some(label),
                options,
                selected,
                onselect,
            }
        }
    }
}
