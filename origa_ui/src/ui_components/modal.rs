use leptos::prelude::*;
use leptos::task::spawn_local_scoped_with_cancellation;
use leptos_use::use_event_listener;

#[component]
pub fn Modal(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional)] on_close: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new(children);
    let is_closing = RwSignal::new(false);
    let on_close_clone = on_close;

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let test_id_close = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-close", val))
        }
    };

    let test_id_backdrop = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-backdrop", val))
        }
    };

    let close_modal_anim = move |ev: leptos::ev::MouseEvent| {
        is_closing.set(true);
        let is_open_clone = is_open;
        let on_close_inner = on_close_clone;
        let ev_clone = ev.clone();
        spawn_local_scoped_with_cancellation(async move {
            gloo_timers::future::TimeoutFuture::new(250).await;
            is_open_clone.set(false);
            is_closing.set(false);
            if let Some(on_close) = on_close_inner {
                on_close.run(ev_clone);
            }
        });
    };

    let cleanup = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" {
                is_closing.set(true);
                let is_open_clone = is_open;
                let on_close_inner = on_close_clone;
                spawn_local_scoped_with_cancellation(async move {
                    gloo_timers::future::TimeoutFuture::new(250).await;
                    is_open_clone.set(false);
                    is_closing.set(false);
                    if let Some(on_close) = on_close_inner {
                        if let Ok(ev) = leptos::ev::MouseEvent::new("click") {
                            on_close.run(ev);
                        }
                    }
                });
            }
        },
    );
    on_cleanup(move || drop(cleanup));

    let backdrop_class = move || {
        if is_closing.get() {
            "modal-backdrop anima-backdrop-exit"
        } else {
            "modal-backdrop anima-backdrop-enter"
        }
    };

    let modal_class = move || {
        if is_closing.get() {
            "modal-content anima-modal-exit"
        } else {
            "modal-content anima-modal-enter"
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <>
                <div
                    class=backdrop_class
                    on:click=close_modal_anim
                    data-testid=test_id_backdrop
                ></div>
                <div class=modal_class data-testid=test_id_val>
                    <div class="modal-header">
                        <div>
                            <h3 class="drawer-header">{move || title.get()}</h3>
                        </div>
                        <button
                            on:click=close_modal_anim
                            class="modal-close-btn"
                            data-testid=test_id_close
                        >
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <path d="M18 6L6 18M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    {move || children.with_value(|c| c())}
                </div>
            </>
        </Show>
    }
}
