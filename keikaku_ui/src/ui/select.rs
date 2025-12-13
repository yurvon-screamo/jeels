use dioxus::prelude::*;

#[component]
pub fn Select<T: Clone + PartialEq + std::fmt::Display + 'static>(
    options: Vec<T>,
    selected: ReadSignal<Option<T>>,
    onselect: EventHandler<T>,
    label: Option<String>,
) -> Element {
    let mut is_open = use_signal(|| false);
    let options_signal = use_signal(move || options);

    rsx! {
        div { class: "relative w-full z-30",
            if let Some(label_text) = label {
                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1", {label_text} }
            }
            button {
                class: "relative w-full px-5 py-4 rounded-xl bg-slate-50 text-left cursor-pointer outline-none focus:bg-white focus:ring-4 focus:ring-purple-50 focus:shadow-lg transition-all duration-300 group",
                onclick: move |_| {
                    is_open.set(!is_open());
                },
                span { class: "font-medium text-slate-700",
                    {
                        if let Some(selected_val) = selected() {
                            format!("{}", selected_val)
                        } else {
                            "Select...".to_string()
                        }
                    }
                }
                div {
                    class: {
                        if is_open() {
                            "absolute right-5 top-1/2 -translate-y-1/2 pointer-events-none transition-transform duration-300 rotate-180"
                        } else {
                            "absolute right-5 top-1/2 -translate-y-1/2 pointer-events-none transition-transform duration-300"
                        }
                    },
                    svg {
                        class: "w-5 h-5 text-slate-400",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M19 9l-7 7-7-7",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }
            }
            if is_open() {
                div { class: "absolute top-full left-0 w-full mt-2 bg-white rounded-2xl shadow-soft-hover border border-slate-100 z-40",
                    ul { class: "flex flex-col p-2",
                        for idx in 0..options_signal().len() {
                            li {
                                key: "{idx}",
                                class: "px-4 py-3 rounded-xl text-slate-600 font-medium hover:bg-purple-50 hover:text-accent-purple cursor-pointer transition-colors duration-200",
                                onclick: move |_| {
                                    let options = options_signal();
                                    if let Some(opt) = options.get(idx).cloned() {
                                        onselect.call(opt);
                                        is_open.set(false);
                                    }
                                },
                                {format!("{}", options_signal()[idx])}
                            }
                        }
                    }
                }
            }
        }
    }
}
