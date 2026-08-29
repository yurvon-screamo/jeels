use crate::i18n::use_i18n;
use leptos::prelude::*;

use super::ConfirmModal;

/// Подтверждение удаления карты — тонкая обёртка общего
/// [`ConfirmModal`] с текстами удаления (API и testid-суффиксы прежние).
#[component]
pub fn DeleteConfirmModal(
    #[prop(optional, into)] test_id: Signal<String>,
    is_open: RwSignal<bool>,
    is_deleting: Signal<bool>,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <ConfirmModal
            test_id=test_id
            is_open=is_open
            is_busy=is_deleting
            title=Signal::derive(move || {
                i18n.get_keys().ui().delete_card().inner().to_string()
            })
            message=Signal::derive(move || {
                i18n.get_keys().ui().delete_card_message().inner().to_string()
            })
            confirm_label=Signal::derive(move || {
                i18n.get_keys().common().delete().inner().to_string()
            })
            on_confirm=on_confirm
            on_close=on_close
        />
    }
}
