use crate::components::{Button, ButtonVariant, Paragraph};
use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            class: "bg-bg min-h-screen flex items-center justify-center p-8",
            // Background blobs
            div {
                class: "fixed top-[-20%] left-[-10%] w-[700px] h-[700px] bg-purple-300/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[8000ms] pointer-events-none"
            }
            div {
                class: "fixed top-[10%] right-[-20%] w-[600px] h-[600px] bg-accent-pink/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[10000ms] pointer-events-none"
            }
            div {
                class: "fixed bottom-[-10%] right-[10%] w-[600px] h-[600px] bg-accent-cyan/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[12000ms] pointer-events-none"
            }
            
            div {
                class: "max-w-4xl mx-auto relative z-10 text-center animate-enter",
                h1 {
                    class: "text-5xl md:text-6xl mb-4 font-bold text-slate-800",
                    "Welcome to "
                    span {
                        class: "bg-clip-text text-transparent bg-rainbow-vibrant",
                        "Keikaku"
                    }
                }
                Paragraph {
                    class: Some("text-lg mb-8 text-text-muted".to_string()),
                    "A beautiful UI component library built with Dioxus and Tailwind CSS"
                }
                div {
                    class: "flex flex-col sm:flex-row gap-4 justify-center items-center",
                    Button {
                        variant: ButtonVariant::Rainbow,
                        onclick: move |_| {},
                        class: Some("w-full sm:w-auto".to_string()),
                        "Get Started"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {},
                        class: Some("w-full sm:w-auto".to_string()),
                        "View Showcase"
                    }
                }
                div {
                    class: "mt-12 grid grid-cols-2 md:grid-cols-3 gap-4",
                    a {
                        href: "https://dioxuslabs.com/learn/0.7/",
                        class: "px-4 py-3 rounded-xl bg-surface shadow-soft hover:shadow-soft-hover transition-all duration-300 text-text-main hover:text-accent-pink",
                        "📚 Learn Dioxus"
                    }
                    a {
                        href: "https://dioxuslabs.com/awesome",
                        class: "px-4 py-3 rounded-xl bg-surface shadow-soft hover:shadow-soft-hover transition-all duration-300 text-text-main hover:text-accent-purple",
                        "🚀 Awesome Dioxus"
                    }
                    a {
                        href: "https://github.com/dioxus-community/",
                        class: "px-4 py-3 rounded-xl bg-surface shadow-soft hover:shadow-soft-hover transition-all duration-300 text-text-main hover:text-accent-cyan",
                        "📡 Community"
                    }
                }
            }
        }
    }
}
