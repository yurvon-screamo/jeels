use dioxus::prelude::*;

#[component]
pub fn Switch(
    checked: ReadSignal<bool>,
    onchange: EventHandler<bool>,
    label: Option<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-3 rounded-xl bg-slate-50",
            if let Some(label_text) = label {
                span { class: "text-sm font-bold text-slate-600", {label_text} }
            }
            label { class: "relative inline-flex items-center cursor-pointer",
                input {
                    r#type: "checkbox",
                    checked: checked(),
                    class: "sr-only peer bg-rainbow-switch",
                    onchange: move |e| {
                        onchange.call(e.checked());
                    },
                }
                div { class: if checked() { "w-12 h-7 rounded-full transition-all duration-300 ease-smooth relative overflow-hidden bg-rainbow-vibrant shadow-glow" } else { "w-12 h-7 rounded-full transition-all duration-300 ease-smooth relative overflow-hidden bg-slate-200" } }
                div { class: "absolute left-[4px] top-[4px] bg-white w-5 h-5 rounded-full shadow-md transition-transform duration-300 ease-elastic peer-checked:translate-x-5 z-10" }
            }
        }
    }
}

#[component]
pub fn Checkbox(
    checked: ReadSignal<bool>,
    onchange: EventHandler<bool>,
    label: Option<String>,
) -> Element {
    rsx! {
        label { class: "flex items-center gap-3 cursor-pointer group",
            input {
                r#type: "checkbox",
                checked: checked(),
                class: "custom-check appearance-none w-5 h-5 rounded-lg border-2 border-slate-300 bg-white transition-all duration-200 ease-elastic select-none",
                onchange: move |e| {
                    onchange.call(e.checked());
                },
            }
            if let Some(label_text) = label {
                span { class: "text-sm font-medium text-slate-600 group-hover:text-accent-pink transition-colors",
                    {label_text}
                }
            }
        }
    }
}

#[component]
pub fn Radio(
    checked: ReadSignal<bool>,
    onchange: EventHandler<bool>,
    name: String,
    label: Option<String>,
) -> Element {
    rsx! {
        label { class: "flex items-center gap-3 cursor-pointer group",
            div { class: "relative flex items-center",
                input {
                    r#type: "radio",
                    name: name.clone(),
                    checked: checked(),
                    class: "custom-radio appearance-none w-5 h-5 rounded-full border-2 border-slate-300 checked:border-accent-pink transition-colors",
                    onchange: move |e| {
                        onchange.call(e.checked());
                    },
                }
                div { class: "absolute inset-0 m-auto w-2.5 h-2.5 rounded-full bg-accent-pink scale-0 peer-checked:scale-100 transition-transform duration-300 ease-elastic" }
            }
            if let Some(label_text) = label {
                span { class: "text-sm font-medium text-slate-600", {label_text} }
            }
        }
    }
}
