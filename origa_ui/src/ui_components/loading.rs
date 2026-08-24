use leptos::prelude::*;

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] size: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let size_class = Signal::derive(move || {
        match size.get().as_str() {
            "sm" => "spinner-sm",
            "lg" => "spinner-lg",
            _ => "",
        }
        .to_string()
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class=move || format!("spinner {} {}", size_class.get(), class.get())></div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(into)] message: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    /// Optional `(completed, total)` resource-download progress shown as a
    /// bar under the message (Guideline 4.2.3(ii)). `None` keeps the plain
    /// spinner-only overlay used by every other call site.
    #[prop(optional, into)]
    progress: Signal<Option<(u32, u32)>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let bar_value = RwSignal::new(0u32);
    let bar_max = RwSignal::new(0u32);
    let bar_visible = RwSignal::new(false);
    Effect::new(move |_| match progress.get() {
        Some((completed, total)) if total > 0 => {
            bar_value.set(completed.min(total));
            bar_max.set(total);
            bar_visible.set(true);
        },
        _ => bar_visible.set(false),
    });

    view! {
        <div data-testid=test_id_val class=move || format!("loading-overlay anima-page-fade {}", class.get())>
            <Spinner class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "".to_string()) test_id="loading-spinner" />
            <p class="loading-overlay-message">{move || message.get()}</p>
            {move || {
                if !bar_visible.get() {
                    return None;
                }
                // Same track/fill markup as `ProgressBar`, inlined because the
                // component takes `RwSignal` props while this overlay derives
                // values from a parent-owned signal.
                let percentage = (bar_value.get() as f64 / bar_max.get() as f64 * 100.0)
                    .min(100.0);
                Some(view! {
                    <div class="loading-overlay-progress" data-testid="loading-progress-bar">
                        <div class="progress-track">
                            <div
                                class="progress-fill"
                                style=format!("--progress-width: {percentage:.0}%")
                            ></div>
                        </div>
                    </div>
                })
            }}
        </div>
    }
}
