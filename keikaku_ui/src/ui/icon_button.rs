use dioxus::prelude::*;

#[component]
pub fn IconButton(
    icon: Element,
    onclick: Option<EventHandler<MouseEvent>>,
    class: Option<String>,
    rounded: Option<bool>,
) -> Element {
    let is_rounded = rounded.unwrap_or(false);
    let shape_class = if is_rounded {
        "rounded-full"
    } else {
        "rounded-xl"
    };
    let class_str = class.unwrap_or_default();

    rsx! {
        button {
            class: "w-12 h-12 {shape_class} bg-rainbow-vibrant text-white flex items-center justify-center shadow-md shadow-accent-pink/15 hover:scale-110 hover:shadow-glow active:scale-95 transition-all duration-300 ease-elastic {class_str}",
            onclick: move |e| {
                if let Some(handler) = onclick.as_ref() {
                    handler.call(e);
                }
            },
            {icon}
        }
    }
}
