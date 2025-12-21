use dioxus::prelude::*;

/// InfoGrid - адаптивный компонент сетки для отображения элементов в колонках.
/// Автоматически перестраивается: 1 колонка на мобильных, 2 колонки на десктопе.
#[component]
pub fn InfoGrid(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();

    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-3 {class_str}", {children} }
    }
}
