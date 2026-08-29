use leptos::prelude::*;
use leptos_use::use_event_listener;

use crate::utils::scroll_lock;

#[component]
pub fn Drawer(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional)] on_close: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional)] action_button: Option<ChildrenFn>,
    /// Pinned action bar rendered OUTSIDE the scrollable body (a sibling of
    /// `.drawer-body`, inside `.drawer-content`). Use for Cancel/Submit rows
    /// that must stay reachable while long content scrolls — the footer needs
    /// no background of its own because it never overlaps scrolling content.
    #[prop(optional)]
    footer: Option<ChildrenFn>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    // Scroll lock is held exactly once per open transition: `hold` guards
    // against double-lock on repeated Effect runs, and both close paths
    // (signal transition + component unmount) release it idempotently.
    // A signal (not a plain Cell) so both closures stay shareable.
    let hold = RwSignal::new(false);
    let hold_for_effect = hold;
    Effect::new(move |_| {
        if is_open.get() {
            if !hold_for_effect.get() {
                hold_for_effect.set(true);
                scroll_lock::lock_scroll();
            }
        } else if hold_for_effect.get() {
            hold_for_effect.set(false);
            scroll_lock::unlock_scroll();
        }
    });
    on_cleanup(move || {
        if hold.get() {
            scroll_lock::unlock_scroll();
        }
    });

    let close_drawer = move |ev: leptos::ev::MouseEvent| {
        is_open.set(false);
        if let Some(on_close) = on_close {
            on_close.run(ev);
        }
    };

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

    let _ = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" {
                is_open.set(false);
            }
        },
    );

    let children = StoredValue::new(children);
    let action_button = StoredValue::new(action_button);
    let footer = StoredValue::new(footer);

    view! {
        <Show when=move || is_open.get()>
            <>
                {/* Use the same backdrop as the modal for consistency */}
                <div
                    class="modal-backdrop anima-backdrop-enter"
                    on:click=close_drawer
                    data-testid=test_id_backdrop
                ></div>

                <div class="drawer-content" data-testid=test_id_val>
                    {/* Header: fixed */}
                    <div class="modal-header">
                        <div>
                            <h3 class="drawer-header">{move || title.get()}</h3>
                        </div>
                        <div class="flex items-center gap-2">
                            {move || action_button.with_value(|b| b.as_ref().map(|c| c()))}
                            <button
                                on:click=close_drawer
                                class="modal-close-btn"
                                data-testid=test_id_close
                            >
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <path d="M18 6L6 18M6 6l12 12" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    {/* Body: scrollable content */}
                    <div class="drawer-body">
                        {move || children.with_value(|c| c())}
                    </div>

                    {/* Footer: pinned outside the scroll area. The wrapper
                        div exists only when the footer prop is provided,
                        so drawers without a footer render unchanged. */}
                    {move || {
                        footer.with_value(|f| {
                            f.as_ref().map(|c| {
                                view! {
                                    <div class="drawer-footer">
                                        {c()}
                                    </div>
                                }
                                .into_any()
                            })
                        })
                    }}
                </div>
            </>
        </Show>
    }
}
