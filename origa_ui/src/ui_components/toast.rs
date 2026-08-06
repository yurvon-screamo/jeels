use leptos::prelude::*;
use leptos::task::spawn_local_scoped_with_cancellation;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ToastType {
    #[default]
    Info,
    Success,

    #[expect(dead_code, reason = "design system API")]
    Warning,
    Error,
}

#[derive(Clone)]
pub struct ToastData {
    pub id: usize,
    pub toast_type: ToastType,
    pub title: String,
    pub message: String,
    pub duration_ms: Option<u64>,
    pub closable: bool,
}

impl Default for ToastData {
    fn default() -> Self {
        Self {
            id: 0,
            toast_type: ToastType::Info,
            title: String::new(),
            message: String::new(),
            duration_ms: None,
            closable: true,
        }
    }
}

#[component]
pub fn Toast(
    toast: ToastData,
    on_close: Callback<usize>,
    #[prop(optional)] duration_ms: u64,
    #[prop(optional)] is_closing: bool,
) -> impl IntoView {
    let toast_id = toast.id;
    let actual_duration = toast.duration_ms.unwrap_or(duration_ms);
    let has_duration = actual_duration > 0;

    if has_duration {
        spawn_local_scoped_with_cancellation(async move {
            gloo_timers::future::TimeoutFuture::new(actual_duration as u32).await;
            on_close.run(toast_id);
        });
    }

    view! {
        <div
            class=move || {
                let toast_class = match toast.toast_type {
                    ToastType::Info => "toast-info",
                    ToastType::Success => "toast-success",
                    ToastType::Warning => "toast-warning",
                    ToastType::Error => "toast-error",
                };
                let anim_class = if is_closing {
                    "anima-toast-exit"
                } else {
                    "anima-toast-bounce"
                };
                format!("toast {} {}", anim_class, toast_class)
            }
            data-testid=format!("toast-{}", toast.id)
        >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                {move || match toast.toast_type {
                    ToastType::Success => view! {
                        <path d="M20 6L9 17l-5-5" />
                    }.into_any(),
                    ToastType::Warning => view! {
                        <path d="M12 9v4m0 4h.01M12 3l9 18H3L12 3z" />
                    }.into_any(),
                    ToastType::Error => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M15 9l-6 6m0-6l6 6" /></>
                    }.into_any(),
                    ToastType::Info => view! {
                        <><circle cx="12" cy="12" r="10" />
                        <path d="M12 16v-4m0-4h.01" /></>
                    }.into_any(),
                }}
            </svg>
            <div class="toast-content">
                <p class="toast-title">{toast.title}</p>
                <p class="toast-message">{toast.message}</p>
            </div>
            {move || if toast.closable {
                view! {
                    <button
                        class="toast-close"
                        data-testid=format!("toast-{}-close", toast_id)
                        on:click=move |_| on_close.run(toast_id)
                    >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M18 6L6 18M6 6l12 12" />
                        </svg>
                    </button>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn ToastContainer(
    toasts: RwSignal<Vec<ToastData>>,
    #[prop(optional)] duration_ms: u64,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let closing_toasts = RwSignal::new(HashMap::<usize, bool>::new());

    let on_close = Callback::new(move |id: usize| {
        let toasts_clone = toasts;
        closing_toasts.update(|c| {
            c.insert(id, true);
        });
        spawn_local_scoped_with_cancellation(async move {
            gloo_timers::future::TimeoutFuture::new(200).await;
            toasts_clone.update(|t| t.retain(|toast| toast.id != id));
            closing_toasts.update(|c| {
                c.remove(&id);
            });
        });
    });

    let is_closing = move |id: usize| closing_toasts.with(|c| c.get(&id).copied().unwrap_or(false));

    view! {
        <div class="toast-container" data-testid=test_id_val>
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    view! {
                        <Toast toast=toast.clone() on_close=on_close duration_ms=duration_ms is_closing=is_closing(toast.id) />
                    }
                }
            />
        </div>
    }
}
