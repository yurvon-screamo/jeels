use crate::i18n::{t, use_i18n};
use leptos::prelude::*;

use super::{Button, ButtonVariant, Modal, Spinner, Text, TextSize, TypographyVariant};

/// Общий паттерн подтверждения действия: Modal + Ghost-отмена +
/// Filled-подтверждение (выделен из DeleteConfirmModal; занятость —
/// Spinner и залоченные кнопки).
#[component]
pub fn ConfirmModal(
    #[prop(optional, into)] test_id: Signal<String>,
    is_open: RwSignal<bool>,
    #[prop(optional)] is_busy: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: Signal<String>,
    #[prop(into)] confirm_label: Signal<String>,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let cancel_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{val}-cancel")
        }
    });

    let confirm_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{val}-confirm")
        }
    });

    view! {
        <Modal test_id=test_id is_open=is_open title=title>
            <div class="confirm-modal">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {move || message.get()}
                </Text>
                <div class="confirm-modal-actions">
                    <Button
                        test_id=cancel_test_id
                        variant=Signal::derive(|| ButtonVariant::Ghost)
                        disabled=is_busy
                        on_click=Callback::new(move |_| on_close.run(()))
                    >
                        {t!(i18n, common.cancel)}
                    </Button>
                    <Button
                        test_id=confirm_test_id
                        variant=Signal::derive(|| ButtonVariant::Filled)
                        disabled=is_busy
                        on_click=Callback::new(move |_| {
                            on_confirm.run(());
                        })
                    >
                        {move || if is_busy.get() {
                            view! { <Spinner /> }.into_any()
                        } else {
                            let label = confirm_label.get();
                            label.into_any()
                        }}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
