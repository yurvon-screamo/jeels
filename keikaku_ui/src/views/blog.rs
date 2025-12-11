use crate::components::{Button, ButtonVariant, Card, H1, Paragraph};
use crate::Route;
use dioxus::prelude::*;

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            class: "bg-bg min-h-screen p-8",
            // Background blobs
            div {
                class: "fixed top-[-20%] left-[-10%] w-[700px] h-[700px] bg-purple-300/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[8000ms] pointer-events-none"
            }
            div {
                class: "fixed top-[10%] right-[-20%] w-[600px] h-[600px] bg-accent-pink/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[10000ms] pointer-events-none"
            }
            
            div {
                class: "max-w-4xl mx-auto relative z-10",
                Card {
                    delay: Some("100".to_string()),
                    div {
                        class: "space-y-6",
                        H1 {
                            "Blog Post #{id}"
                        }
                        Paragraph {
                            class: Some("text-base".to_string()),
                            "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components. This demonstrates the power of type-safe routing in Dioxus applications."
                        }
                        Paragraph {
                            class: Some("text-base".to_string()),
                            "The router automatically handles URL synchronization and component rendering based on the current route. You can navigate between routes using the Link component, which ensures type safety at compile time."
                        }
                        div {
                            class: "flex items-center gap-4 pt-6 border-t border-slate-100",
                            Link {
                                to: Route::Blog { id: id - 1 },
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| {},
                                    class: Some("w-auto".to_string()),
                                    "← Previous"
                                }
                            }
                            Link {
                                to: Route::Blog { id: id + 1 },
                                Button {
                                    variant: ButtonVariant::Rainbow,
                                    onclick: move |_| {},
                                    class: Some("w-auto".to_string()),
                                    "Next →"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
