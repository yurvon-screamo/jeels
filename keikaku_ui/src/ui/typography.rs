use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum Size {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[component]
pub fn H1(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h1 { class: "text-3xl font-bold mb-1 text-slate-800 {class_str}", {children} }
    }
}

#[component]
pub fn H2(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h2 { class: "text-2xl font-bold mb-1 text-slate-700 {class_str}", {children} }
    }
}

#[component]
pub fn H3(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h3 { class: "text-xl font-bold mb-1 text-slate-600 {class_str}", {children} }
    }
}

#[component]
pub fn H4(class: Option<String>, children: Element) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        h4 { class: "text-lg font-bold mb-1 text-slate-500 {class_str}", {children} }
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
pub fn Tag(size: Option<Size>, children: Element) -> Element {
    let size_class = match size.unwrap_or(Size::Small) {
        Size::Small => "text-xs",
        Size::Medium => "text-sm",
        Size::Large => "text-base",
        Size::ExtraLarge => "text-lg",
    };

    rsx! {
        span { class: "inline-block px-3 py-2 rounded-lg bg-gradient-to-r from-pink-50 to-cyan-50 text-accent-purple font-bold {size_class}",
            {children}
        }
    }
}
