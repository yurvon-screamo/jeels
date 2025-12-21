use dioxus::prelude::*;
use dioxus_core::AttributeValue;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Destructive => "destructive",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    // If the caller passes `class`, it would otherwise override our default class due to `..attributes`.
    // We always want `.button` styles to apply, so we merge classes and remove `class` from attributes.
    let mut merged_attributes: Vec<Attribute> = Vec::with_capacity(attributes.len());
    let mut user_class = String::new();

    for attr in attributes {
        if attr.name == "class" {
            // Best-effort extraction of the class string.
            // Dioxus typically stores it as text.
            if let AttributeValue::Text(text) = attr.value {
                user_class = text;
            }
        } else {
            merged_attributes.push(attr);
        }
    }

    let class = if user_class.trim().is_empty() {
        "button".to_string()
    } else {
        format!("button {}", user_class.trim())
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        button {
            class: "{class}",
            "data-style": variant.class(),
            onclick: move |event| {
                if let Some(f) = &onclick {
                    f.call(event);
                }
            },
            onmousedown: move |event| {
                if let Some(f) = &onmousedown {
                    f.call(event);
                }
            },
            onmouseup: move |event| {
                if let Some(f) = &onmouseup {
                    f.call(event);
                }
            },
            ..merged_attributes,
            {children}
        }
    }
}
