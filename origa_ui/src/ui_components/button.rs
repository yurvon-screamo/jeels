use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Filled,
    Olive,
    Ghost,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonSize {
    #[default]
    Default,
    Small,

    #[expect(dead_code, reason = "design system API")]
    Large,
}

#[component]
pub fn Button(
    #[prop(optional, into)] variant: Signal<ButtonVariant>,
    #[prop(optional, into)] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] loading: Signal<bool>,
    #[prop(optional, into)] button_type: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <button
            type=move || {
                let t = button_type.get();
                if t.is_empty() { "button".to_string() } else { t }
            }
            class=move || {
                let v = match variant.get() {
                    ButtonVariant::Default => "",
                    ButtonVariant::Filled => "btn-filled",
                    ButtonVariant::Olive => "btn-olive",
                    ButtonVariant::Ghost => "btn-ghost",
                };
                let s = match size.get() {
                    ButtonSize::Default => "",
                    ButtonSize::Small => "btn-sm",
                    ButtonSize::Large => "btn-lg",
                };
                let loading_class = if loading.get() { "btn-loading" } else { "" };
                let focus_ring = "anima-focus-ring";
                let btn_press = "anima-press";
                format!("btn {} {} {} {} {} {}", v, s, class.get(), loading_class, focus_ring, btn_press)
            }
            disabled=move || disabled.get() || loading.get()
            data-testid=test_id_val
            on:click=move |ev| {
                if let Some(on_click) = on_click {
                    on_click.run(ev);
                }
            }
        >
            <Show when=move || loading.get()>
                <span class="btn-spinner" data-testid="btn-spinner"></span>
            </Show>
            <span class=move || if loading.get() { "btn-text btn-text-hidden" } else { "btn-text" }>
                {children()}
            </span>
        </button>
    }
}
