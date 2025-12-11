use crate::components::*;
use dioxus::prelude::*;

#[component]
pub fn Showcase() -> Element {
    let mut email = use_signal(|| String::new());
    let mut search = use_signal(|| String::new());
    let mut message = use_signal(|| String::new());
    let mut switch_checked = use_signal(|| false);
    let mut checkbox1 = use_signal(|| false);
    let mut checkbox2 = use_signal(|| false);
    let mut radio_plan = use_signal(|| "free".to_string());
    let mut selected_option = use_signal(|| Option::<String>::None);

    let options = vec![
        "Design System".to_string(),
        "Development".to_string(),
        "Marketing Strategy".to_string(),
    ];

    rsx! {
        div { class: "bg-bg text-text-main p-8 min-h-screen selection:bg-accent-pink/30 selection:text-text-main",
            // Background blobs
            div { class: "fixed top-[-20%] left-[-10%] w-[700px] h-[700px] bg-purple-300/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[8000ms] pointer-events-none" }
            div { class: "fixed top-[10%] right-[-20%] w-[600px] h-[600px] bg-accent-pink/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[10000ms] pointer-events-none" }
            div { class: "fixed bottom-[-10%] right-[10%] w-[600px] h-[600px] bg-accent-cyan/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[12000ms] pointer-events-none" }

            div { class: "max-w-7xl mx-auto relative z-10",
                header { class: "mb-12 animate-enter",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-2 tracking-tight",
                        "Rainbow "
                        span { class: "bg-clip-text text-transparent bg-rainbow-vibrant",
                            "Uwuwu UI Kit"
                        }
                    }
                    p { class: "text-text-muted text-lg",
                        "Полная библиотека компонентов Uwuwu UI"
                    }
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8",
                    // Typography Card
                    Card { delay: Some("100".to_string()),
                        div {
                            span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-4 block",
                                "01. Typography & Card"
                            }
                            H1 { "Headline H1" }
                            H2 { "Headline H2" }
                            H3 { "Headline H3" }
                            hr { class: "border-slate-100 my-4" }
                            Paragraph {
                                "Шрифты и цвета адаптированы под мягкую, \"зефирную\" палитру. Черный цвет заменен на глубокие оттенки серого для снижения контраста."
                            }
                        }
                        div { class: "mt-6 pt-4 border-t border-slate-50",
                            Tag { "Rainbow Tag" }
                        }
                    }

                    // Buttons Card
                    Card { delay: Some("200".to_string()),
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "02. Buttons"
                        }
                        div { class: "flex flex-col gap-4",
                            Button {
                                variant: ButtonVariant::Rainbow,
                                onclick: move |_| {},
                                "Rainbow Action"
                            }
                            Button {
                                variant: ButtonVariant::Pearlescent,
                                onclick: move |_| {},
                                "Pearlescent Soft"
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {},
                                "Outline Button"
                            }
                            div { class: "flex gap-4 justify-center mt-2",
                                IconButton {
                                    icon: rsx! {
                                        svg {
                                            class: "w-6 h-6",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                            }
                                        }
                                    },
                                    onclick: move |_| {},
                                    class: Some("bg-white text-slate-400 hover:text-accent-pink".to_string()),
                                }
                                IconButton {
                                    icon: rsx! {
                                        svg {
                                            class: "w-5 h-5",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M14 5l7 7m0 0l-7 7m7-7H3",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                            }
                                        }
                                    },
                                    onclick: move |_| {},
                                    rounded: Some(true),
                                }
                            }
                        }
                    }

                    // Inputs Card
                    Card { delay: Some("300".to_string()),
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "03. Inputs & Textarea"
                        }
                        div { class: "space-y-5",
                            TextInput {
                                label: Some("EMAIL".to_string()),
                                placeholder: Some("hello@maple.com".to_string()),
                                value: Some(email),
                                oninput: move |e: Event<FormData>| {
                                    email.set(e.value());
                                },
                            }
                            SearchInput {
                                label: Some("SEARCH".to_string()),
                                placeholder: Some("Find component...".to_string()),
                                value: Some(search),
                                oninput: move |e: Event<FormData>| {
                                    search.set(e.value());
                                },
                            }
                            Textarea {
                                label: Some("MESSAGE".to_string()),
                                placeholder: Some("Type something nice...".to_string()),
                                value: Some(message),
                                oninput: move |e: Event<FormData>| {
                                    message.set(e.value());
                                },
                            }
                        }
                    }

                    // Toggles Card
                    Card { delay: Some("400".to_string()),
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "04. Toggles & Controls"
                        }
                        div { class: "space-y-6",
                            Switch {
                                checked: switch_checked,
                                onchange: move |val| {
                                    switch_checked.set(val);
                                },
                                label: Some("Rainbow Mode".to_string()),
                            }
                            hr { class: "border-slate-100" }
                            div { class: "space-y-3",
                                Checkbox {
                                    checked: checkbox1,
                                    onchange: move |val| {
                                        checkbox1.set(val);
                                    },
                                    label: Some("Soft notifications".to_string()),
                                }
                                Checkbox {
                                    checked: checkbox2,
                                    onchange: move |val| {
                                        checkbox2.set(val);
                                    },
                                    label: Some("Holographic newsletter".to_string()),
                                }
                            }
                            div { class: "space-y-3",
                                Radio {
                                    checked: use_signal(|| radio_plan() == "free"),
                                    onchange: move |_| {
                                        radio_plan.set("free".to_string());
                                    },
                                    name: "plan".to_string(),
                                    label: Some("Free Plan".to_string()),
                                }
                                Radio {
                                    checked: use_signal(|| radio_plan() == "pro"),
                                    onchange: move |_| {
                                        radio_plan.set("pro".to_string());
                                    },
                                    name: "plan".to_string(),
                                    label: Some("Pro Plan".to_string()),
                                }
                            }
                        }
                    }

                    // Select Card
                    Card {
                        delay: Some("500".to_string()),
                        class: Some("overflow-visible".to_string()),
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "05. Selects & Dropdowns"
                        }
                        div { class: "space-y-6",
                            Select {
                                options: options.clone(),
                                selected: selected_option,
                                onselect: move |val| {
                                    selected_option.set(Some(val));
                                },
                                label: Some("CATEGORY".to_string()),
                            }
                        }
                    }

                    // Rainbow Composition Card
                    div { class: "bg-rainbow-soft rounded-[2rem] p-8 shadow-xl shadow-accent-purple/20 text-white animate-enter delay-500 relative overflow-hidden group",
                        div { class: "absolute top-0 right-0 w-40 h-40 bg-white/20 opacity-50 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2 group-hover:scale-125 transition-transform duration-700 mix-blend-overlay" }
                        div { class: "absolute bottom-0 left-0 w-32 h-32 bg-accent-cyan/30 opacity-50 rounded-full blur-2xl translate-y-1/2 -translate-x-1/2 group-hover:scale-125 transition-transform duration-700 mix-blend-overlay" }
                        span { class: "text-xs font-bold text-white/70 uppercase tracking-widest mb-6 block relative z-10",
                            "06. Rainbow Composition"
                        }
                        h3 { class: "text-2xl font-bold mb-6 relative z-10 text-white",
                            "Join the Holographic Waitlist"
                        }
                        form { class: "space-y-4 relative z-10",
                            div { class: "grid grid-cols-2 gap-4",
                                input {
                                    class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                    placeholder: "Name",
                                    r#type: "text",
                                }
                                input {
                                    class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                    placeholder: "Surname",
                                    r#type: "text",
                                }
                            }
                            input {
                                class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                placeholder: "email@address.com",
                                r#type: "email",
                            }
                            button {
                                class: "w-full py-3.5 rounded-xl bg-white text-accent-purple font-bold shadow-lg hover:text-accent-pink hover:scale-[1.02] active:scale-95 transition-all duration-300 ease-elastic mt-2",
                                r#type: "button",
                                "Get Early Access"
                            }
                        }
                    }
                }
            }
        }
    }
}
