use dioxus::prelude::*;

/// Универсальный компонент сетки для отображения элементов
/// Поддерживает адаптивную раскладку и кастомные классы
#[component]
pub fn Grid(
    /// Количество колонок на разных экранах (sm:md:lg:xl)
    columns: Option<String>,
    /// Расстояние между элементами
    gap: Option<String>,
    /// Дополнительные CSS классы
    class: Option<String>,
    /// Элементы для отображения
    children: Element,
) -> Element {
    let columns_class =
        columns.unwrap_or_else(|| "grid-cols-1 md:grid-cols-2 lg:grid-cols-3".to_string());
    let gap_class = gap.unwrap_or_else(|| "gap-4".to_string());
    let class_str = class.unwrap_or_default();

    rsx! {
        div { class: "grid {columns_class} {gap_class} {class_str}", {children} }
    }
}
