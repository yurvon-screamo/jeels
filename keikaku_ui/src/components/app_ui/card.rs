use dioxus::prelude::*;

#[component]
pub fn Card(class: Option<String>, delay: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();

    // If the caller provided any padding utility, don't force our default padding.
    // This prevents "mystery extra space" when nesting cards.
    let has_padding = class_str.split_whitespace().any(|c| {
        c.starts_with("p-")
            || c.starts_with("px-")
            || c.starts_with("py-")
            || c.starts_with("pt-")
            || c.starts_with("pr-")
            || c.starts_with("pb-")
            || c.starts_with("pl-")
    });
    let padding_class = if has_padding { "" } else { "p-8" };
    let delay_class = delay.map(|d| format!("delay-{}", d)).unwrap_or_default();

    rsx! {
        div { class: "kk-card bg-surface rounded-[2rem] {padding_class} shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter {delay_class} relative overflow-visible {class_str}",
            {children}
        }
    }
}
