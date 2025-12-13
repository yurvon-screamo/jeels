use dioxus::prelude::*;

#[component]
pub fn H1(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h1 { class: "text-3xl font-bold mb-2 text-slate-800 {class_str}", {children} }
    }
}

#[component]
pub fn H2(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h2 { class: "text-2xl font-bold mb-2 text-slate-700 {class_str}", {children} }
    }
}

#[component]
pub fn H3(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h3 { class: "text-xl font-bold mb-4 text-slate-600 {class_str}", {children} }
    }
}

#[component]
pub fn Paragraph(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        p { class: "text-slate-500 leading-relaxed text-sm {class_str}", {children} }
    }
}

#[component]
pub fn Tag(children: Element) -> Element {
    rsx! {
        span { class: "inline-block px-3 py-1 rounded-lg bg-gradient-to-r from-pink-50 to-cyan-50 text-accent-purple text-xs font-bold",
            {children}
        }
    }
}
