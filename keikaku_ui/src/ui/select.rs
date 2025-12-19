use dioxus::prelude::*;
use dioxus_heroicons::{Icon, outline};

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
        div { class: if is_open() { "relative w-full z-50 overflow-visible" } else { "relative w-full z-10 overflow-visible" },
            // Click outside to close the dropdown.
            if is_open() {
                div {
                    // Use inline z-index to avoid relying on tailwind generation and to ensure
                    // the overlay is below the dropdown menu.
                    style: "position: fixed; inset: 0; z-index: 9998; background: transparent;",
                    onmousedown: move |_| is_open.set(false),
                }
            }
            if let Some(label_text) = label {
                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1", {label_text} }
            }
            button {
                class: "relative w-full px-5 py-4 rounded-xl bg-slate-50 text-left cursor-pointer outline-none focus:bg-white focus:ring-4 focus:ring-purple-50 focus:shadow-lg transition-all duration-300 group",
                // Keep the trigger above the click-outside overlay for consistent behavior.
                style: "z-index: 9999;",
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
                    Icon {
                        icon: outline::Shape::ChevronDown,
                        size: 20,
                        class: Some("w-5 h-5 text-slate-400".to_string()),
                    }
                }
            }
            if is_open() {
                div {
                    class: "absolute top-full left-0 w-full mt-2 bg-white rounded-2xl shadow-soft-hover border border-slate-100",
                    // Ensure the dropdown is above the click-outside overlay.
                    style: "z-index: 10000;",
                    ul { class: "flex flex-col p-2",
                        for idx in 0..options_signal().len() {
                            li {
                                key: "{idx}",
                                class: "px-4 py-3 rounded-xl text-slate-600 font-medium hover:bg-purple-50 hover:text-accent-purple cursor-pointer transition-colors duration-200",
                                onmousedown: move |e| {
                                    e.stop_propagation();
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
