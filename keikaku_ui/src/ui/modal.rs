use dioxus::prelude::*;

#[component]
pub fn Modal(title: String, on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div { class: "fixed inset-0 z-50", onclick: move |_| on_close.call(()),
            div {
                class: "absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity duration-300 ease-out",
                style: "animation: fadeIn 0.3s ease-out;",
            }
            div {
                class: "absolute top-0 right-0 h-full w-full max-w-md bg-white shadow-2xl transform transition-transform duration-300 ease-out",
                style: "animation: slideInRight 0.3s ease-out;",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex items-center justify-between p-6 border-b border-slate-200",
                    h2 { class: "text-xl font-bold text-slate-800", {title} }
                    button {
                        class: "w-8 h-8 flex items-center justify-center rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6 overflow-y-auto max-h-[calc(100vh-120px)]", {children} }
            }
        }
        style {
            "
            @keyframes fadeIn {{
                from {{ opacity: 0; }}
                to {{ opacity: 1; }}
            }}
            @keyframes slideInRight {{
                from {{ transform: translateX(100%); }}
                to {{ transform: translateX(0); }}
            }}
            "
        }
    }
}
