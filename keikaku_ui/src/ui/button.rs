use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum ButtonVariant {
    Rainbow,
    Pearlescent,
    Outline,
}

#[component]
pub fn Button(
    variant: ButtonVariant,
    onclick: Option<EventHandler<MouseEvent>>,
    class: Option<String>,
    disabled: Option<bool>,
    children: Element,
) -> Element {
    let class_str = class.unwrap_or_default();
    let base_classes = match variant {
        ButtonVariant::Rainbow => "w-full py-3.5 rounded-xl bg-rainbow-vibrant text-white font-bold shadow-lg shadow-accent-purple/30 hover:scale-[1.02] hover:shadow-accent-pink/40 active:scale-95 transition-all duration-300 ease-elastic relative overflow-hidden group",
        ButtonVariant::Pearlescent => "w-full py-3.5 rounded-xl bg-pink-50 text-accent-pink font-bold hover:bg-cyan-50 hover:text-accent-cyan hover:shadow-soft active:scale-95 transition-all duration-300 ease-elastic",
        ButtonVariant::Outline => "w-full py-3.5 rounded-xl border-2 border-slate-100 text-slate-500 font-bold hover:border-accent-pink hover:text-accent-pink hover:shadow-soft hover:scale-[1.02] active:scale-95 group transition-all duration-300 ease-elastic relative overflow-hidden",
    };

    rsx! {
        button {
            class: "{base_classes} {class_str}",
            disabled: disabled.unwrap_or(false),
            onclick: move |e| {
                if !disabled.unwrap_or(false) {
                    if let Some(handler) = onclick.as_ref() {
                        handler.call(e);
                    }
                }
            },
            if variant == ButtonVariant::Rainbow {
                div { class: "absolute inset-0 flex items-center justify-center pointer-events-none",
                    div { class: "w-[200%] h-[200%] bg-white/20 rounded-full scale-0 group-hover:scale-100 transition-transform duration-500 ease-out" }
                }
                span { class: "relative z-10",
                    div { class: "px-2 py-1 flex items-center justify-center", {children} }
                }
            } else if variant == ButtonVariant::Outline {
                span { class: "relative z-10",
                    div { class: "px-2 py-1 flex items-center justify-center", {children} }
                }
            } else {
                span { class: "relative z-10",
                    div { class: "px-2 py-1 flex items-center justify-center", {children} }
                }
            }
        }
    }
}
