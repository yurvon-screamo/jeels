use dioxus::prelude::*;

#[component]
pub fn Card(
    class: Option<String>,
    delay: Option<String>,
    children: Element,
) -> Element {
    let class_str = class.unwrap_or_default();
    let delay_class = delay.map(|d| format!("delay-{}", d)).unwrap_or_default();
    
    rsx! {
        div {
            class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter {delay_class} relative overflow-hidden {class_str}",
            {children}
        }
    }
}

