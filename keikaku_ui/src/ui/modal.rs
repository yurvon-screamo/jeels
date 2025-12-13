use dioxus::prelude::*;

#[component]
pub fn Modal(title: String, on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),
            div { class: "absolute inset-0 bg-black/50 backdrop-blur-sm" }
            div {
                class: "relative z-10 w-full max-w-lg bg-white rounded-2xl shadow-2xl p-6 max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex items-center justify-between mb-6",
                    h2 { class: "text-xl font-bold text-slate-800", {title} }
                    button {
                        class: "w-8 h-8 flex items-center justify-center rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                {children}
            }
        }
    }
}
