use dioxus::prelude::*;

#[component]
pub fn Icon(
    icon: Element,
    class: Option<String>,
    size: Option<String>,
    color: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();
    let size_class = size.unwrap_or_else(|| "w-6 h-6".to_string());
    let color_class = color.unwrap_or_else(|| "text-current".to_string());

    rsx! {
        div { class: "flex items-center justify-center {size_class} {color_class} {class_str}",
            {icon}
        }
    }
}
