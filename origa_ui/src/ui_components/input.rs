use leptos::either::Either;
use leptos::ev::Event;
use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(optional)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] rows: Signal<Option<usize>>,
    #[prop(optional, into)] maxlength: Signal<Option<usize>>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] input_type: Signal<String>,
    #[prop(optional, into)] autocomplete: Signal<String>,
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] name: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<Event>>,
    #[prop(optional)] on_keydown: Option<Callback<leptos::ev::KeyboardEvent>>,
) -> impl IntoView {
    let input_type = move || {
        let t = input_type.get();
        if t.is_empty() { "text".to_string() } else { t }
    };
    let autocomplete = move || {
        let a = autocomplete.get();
        if a.is_empty() { "off".to_string() } else { a }
    };
    let id_val = move || {
        let val = id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let name_val = move || {
        let val = name.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let maxlength_val = move || maxlength.get();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let full_class = move || {
        let base_class = "input-field";
        let textarea_class = "resize-none";
        let focus_ring = "anima-focus-ring";
        if rows.get().is_some() {
            format!(
                "{} {} {} {}",
                base_class,
                textarea_class,
                class.get(),
                focus_ring
            )
        } else {
            format!("{} {} {}", base_class, class.get(), focus_ring)
        }
    };

    move || {
        if let Some(r) = rows.get() {
            Either::Left(view! {
                <textarea
                    class=full_class
                    placeholder=move || placeholder.get()
                    disabled=move || disabled.get()
                    autocomplete=autocomplete
                    id=id_val
                    name=name_val
                    maxlength=maxlength_val
                    data-testid=test_id_val
                    rows=r
                    bind:value=value
                    on:change=move |ev| {
                        if let Some(on_change) = on_change {
                            on_change.run(ev);
                        }
                    }
                    on:keydown=move |ev| {
                        if let Some(on_keydown) = on_keydown {
                            on_keydown.run(ev);
                        }
                    }
                />
            })
        } else {
            Either::Right(view! {
                <input
                    type=input_type
                    class=full_class
                    placeholder=move || placeholder.get()
                    disabled=move || disabled.get()
                    autocomplete=autocomplete
                    id=id_val
                    name=name_val
                    maxlength=maxlength_val
                    data-testid=test_id_val
                    bind:value=value
                    on:change=move |ev| {
                        if let Some(on_change) = on_change {
                            on_change.run(ev);
                        }
                    }
                    on:keydown=move |ev| {
                        if let Some(on_keydown) = on_keydown {
                            on_keydown.run(ev);
                        }
                    }
                />
            })
        }
    }
}
