use dioxus::prelude::*;

#[component]
pub fn TextInput(
    label: Option<String>,
    placeholder: Option<String>,
    value: Option<Signal<String>>,
    oninput: Option<EventHandler<Event<FormData>>>,
    class: Option<String>,
    r#type: Option<String>,
) -> Element {
    let input_type = r#type.unwrap_or_else(|| "text".to_string());
    let class_str = class.unwrap_or_default();

    rsx! {
        div {
            class: "group",
            if let Some(label_text) = label {
                label {
                    class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-pink transition-colors",
                    {label_text}
                }
            }
            input {
                class: "w-full px-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-pink-200 focus:ring-4 focus:ring-pink-50 focus:outline-none focus:shadow-sm {class_str}",
                placeholder: placeholder.unwrap_or_default(),
                r#type: input_type,
                value: value.as_ref().map(|v| v()),
                oninput: move |e| {
                    if let Some(handler) = oninput.as_ref() {
                        handler.call(e);
                    }
                },
            }
        }
    }
}

#[component]
pub fn SearchInput(
    label: Option<String>,
    placeholder: Option<String>,
    value: Option<Signal<String>>,
    oninput: Option<EventHandler<Event<FormData>>>,
) -> Element {
    rsx! {
        div {
            class: "group relative",
            if let Some(label_text) = label {
                label {
                    class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-cyan transition-colors",
                    {label_text}
                }
            }
            input {
                class: "w-full pl-11 pr-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-cyan-200 focus:ring-4 focus:ring-cyan-50 focus:outline-none",
                placeholder: placeholder.unwrap_or_default(),
                value: value.as_ref().map(|v| v()),
                oninput: move |e| {
                    if let Some(handler) = oninput.as_ref() {
                        handler.call(e);
                    }
                },
            }
            svg {
                class: "w-5 h-5 absolute left-4 top-[38px] text-slate-400 group-focus-within:text-accent-cyan transition-colors",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                }
            }
        }
    }
}

#[component]
pub fn Textarea(
    label: Option<String>,
    placeholder: Option<String>,
    value: Option<Signal<String>>,
    oninput: Option<EventHandler<Event<FormData>>>,
    rows: Option<u32>,
) -> Element {
    let rows_val = rows.unwrap_or(3);

    rsx! {
        div {
            class: "group",
            if let Some(label_text) = label {
                label {
                    class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-purple transition-colors",
                    {label_text}
                }
            }
            textarea {
                class: "w-full px-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-purple-200 focus:ring-4 focus:ring-purple-50 focus:outline-none resize-none",
                placeholder: placeholder.unwrap_or_default(),
                rows: "{rows_val}",
                value: value.as_ref().map(|v| v()),
                oninput: move |e| {
                    if let Some(handler) = oninput.as_ref() {
                        handler.call(e);
                    }
                },
            }
        }
    }
}
