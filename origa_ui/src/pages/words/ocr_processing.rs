use super::ocr_file_utils::{
    base64_decode, calculate_speed_and_eta, execute_ocr, init_ocr_model, is_image_file,
    read_file_as_data_url,
};
use crate::loaders::ModelLoader;
use crate::loaders::ocr_model_loader::ProgressCallback;
use crate::ui_components::{OcrLoadingStage, OcrLoadingState, ProgressInfo};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::ocr::{JapaneseOCRModel, ModelConfig};
use origa::use_cases::ExtractTextFromImageUseCase;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::{debug, error, info};
use web_sys::File;
use web_sys::js_sys::Date;

thread_local! {
    static CACHED_MODEL: RefCell<Option<Rc<JapaneseOCRModel>>> = const { RefCell::new(None) };
    static MODEL_LOADING: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) enum OcrState {
    #[default]
    Idle,
    Ready,
    Processing,
    Error,
}

const MAX_FILE_SIZE_MB: f64 = 10.0;

#[derive(Clone)]
pub(super) struct ProcessContext {
    pub(super) image_preview: RwSignal<Option<String>>,
    pub(super) ocr_state: RwSignal<OcrState>,
    pub(super) ocr_loading_state: OcrLoadingState,
    pub(super) error_message: RwSignal<Option<String>>,
    pub(super) disposed: StoredValue<()>,
}

pub(super) fn handle_ocr_result(
    i18n: &leptos_i18n::I18nContext<crate::i18n::Locale>,
    result: Result<String, String>,
    ctx: &ProcessContext,
    on_text_extracted: &Callback<String>,
) {
    match result {
        Ok(text) => {
            if text.trim().is_empty() {
                let err = i18n
                    .get_keys()
                    .words()
                    .image()
                    .text_not_recognized()
                    .inner()
                    .to_string();
                ctx.error_message.set(Some(err.clone()));
                ctx.ocr_state.set(OcrState::Error);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                    stage: "recognize".to_string(),
                    message: err.to_string(),
                });
            } else {
                info!(text_length = text.len(), "OCR completed successfully");
                ctx.ocr_state.set(OcrState::Ready);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Completed);
                on_text_extracted.run(text);
            }
        },
        Err(e) => {
            error!(error = %e, "OCR failed");
            ctx.error_message.set(Some(e.clone()));
            ctx.ocr_state.set(OcrState::Error);
            ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                stage: "recognize".to_string(),
                message: e,
            });
        },
    }
}

async fn run_ocr_on_data_url(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    data_url: &str,
    ctx: &ProcessContext,
    on_text_extracted: &Callback<String>,
) {
    ctx.ocr_state.set(OcrState::Processing);
    ctx.ocr_loading_state.start_time.set(Some(Date::now()));

    // Yield to the browser so the UI repaints with Processing state before
    // the CPU-bound inference begins. Without this, the single-threaded WASM
    // event loop runs process_image_with_ocr synchronously and the user
    // never sees the loading stepper.
    crate::utils::yield_to_browser().await;

    let result = process_image_with_ocr(data_url, &ctx.ocr_loading_state, &i18n).await;

    if ctx.ocr_loading_state.cancel_requested.get_untracked() {
        return;
    }

    handle_ocr_result(&i18n, result, ctx, on_text_extracted);
}

pub(super) fn process_file(
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    file: File,
    ctx: ProcessContext,
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
) {
    let file_name = file.name();
    debug!(file_name = %file_name, "File selected");

    if !is_image_file(&file) {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .image()
                .not_image()
                .inner()
                .to_string(),
        ));
        return;
    }

    let file_size_mb = file.size() / (1024.0 * 1024.0);
    if file_size_mb > MAX_FILE_SIZE_MB {
        ctx.error_message.set(Some(
            i18n.get_keys()
                .words()
                .image()
                .file_too_large()
                .inner()
                .to_string()
                .replacen("{}", &file_size_mb.to_string(), 1)
                .replacen("{}", &MAX_FILE_SIZE_MB.to_string(), 1),
        ));
        return;
    }

    spawn_local(async move {
        ctx.ocr_loading_state.cancel_requested.set(false);
        match read_file_as_data_url(&file).await {
            Ok(data_url) => {
                if ctx.disposed.is_disposed() {
                    return;
                }
                if ctx.ocr_loading_state.cancel_requested.get_untracked() {
                    return;
                }
                ctx.image_preview.set(Some(data_url.clone()));
                run_ocr_on_data_url(i18n, &data_url, &ctx, &on_text_extracted).await;
            },
            Err(e) => {
                error!(error = %e, "Failed to read file");
                ctx.error_message.set(Some(e.clone()));
                ctx.ocr_state.set(OcrState::Error);
                ctx.ocr_loading_state.stage.set(OcrLoadingStage::Error {
                    stage: "init".to_string(),
                    message: e.clone(),
                });
                on_error.run(e);
            },
        }
    });
}

async fn process_image_with_ocr(
    data_url: &str,
    loading_state: &OcrLoadingState,
    i18n: &leptos_i18n::I18nContext<crate::i18n::Locale>,
) -> Result<String, String> {
    let base64_data = data_url
        .strip_prefix("data:image/")
        .and_then(|s| s.split_once(";base64,"))
        .map(|(_, data)| data)
        .ok_or("Invalid data URL format")?;

    // Native device-ai OCR is primary (no model download); WASM NDLOCR is the
    // fallback. Falls through on any native failure or unavailability.
    if let Some(text) =
        super::ocr_device_ai::recognize_via_device_ai(base64_data, loading_state).await
    {
        return Ok(text);
    }

    let bytes = base64_decode(base64_data)?;

    let model = CACHED_MODEL.with(|cached| cached.borrow().clone());

    let model = match model {
        Some(m) => m,
        None => {
            if MODEL_LOADING.with(|loading| loading.get()) {
                return Err(i18n
                    .get_keys()
                    .words()
                    .image()
                    .model_loading()
                    .inner()
                    .to_string());
            }

            MODEL_LOADING.with(|loading| loading.set(true));
            debug!("Starting OCR model loading");

            let result = async {
                info!("Loading OCR and Layout models");

                let config =
                    ModelConfig::new(crate::core::config::cdn_url("/ndlocr"), "ndlocr-model-");

                let loading_state_ref = *loading_state;
                let progress_callback: ProgressCallback =
                    Rc::new(move |filename, loaded, total| {
                        let stage = loading_state_ref.stage.get_untracked();
                        let start_time = loading_state_ref.start_time.get_untracked();

                        let percent = if total > 0 {
                            ((loaded as f64 / total as f64) * 100.0) as u32
                        } else {
                            0
                        };

                        let (speed_bps, eta_seconds) =
                            calculate_speed_and_eta(start_time, loaded, total);

                        let progress = ProgressInfo {
                            percent,
                            loaded_bytes: loaded,
                            total_bytes: total,
                            speed_bps,
                            eta_seconds,
                        };

                        if filename.contains("deim") {
                            loading_state_ref
                                .stage
                                .set(OcrLoadingStage::DownloadingDeim { progress });
                        } else if filename.contains("parseq") {
                            let current = if filename.contains("parseq-30") {
                                1
                            } else if filename.contains("parseq-50") {
                                2
                            } else if filename.contains("parseq-100") {
                                3
                            } else {
                                match stage {
                                    OcrLoadingStage::DownloadingParseq {
                                        current_model, ..
                                    } => current_model,
                                    _ => 1,
                                }
                            };
                            loading_state_ref
                                .stage
                                .set(OcrLoadingStage::DownloadingParseq {
                                    current_model: current,
                                    progress,
                                });
                        }
                    });

                let loader = ModelLoader::new(config).with_progress_callback(progress_callback);

                loading_state.stage.set(OcrLoadingStage::DownloadingDeim {
                    progress: ProgressInfo::default(),
                });

                let model_files = loader.load_or_download_model().await.map_err(|e| {
                    loading_state.stage.set(OcrLoadingStage::Error {
                        stage: "deim".to_string(),
                        message: format!("Failed to load models: {:?}", e),
                    });
                    format!("Failed to load models: {:?}", e)
                })?;

                if loading_state.cancel_requested.get_untracked() {
                    return Err(i18n
                        .get_keys()
                        .words()
                        .image()
                        .operation_canceled()
                        .inner()
                        .to_string());
                }

                loading_state.stage.set(OcrLoadingStage::Initializing {
                    model_name: "OCR models".to_string(),
                });

                let new_model = init_ocr_model(model_files, loading_state)
                    .await
                    .map_err(|e| e.to_string())?;

                if loading_state.cancel_requested.get_untracked() {
                    return Err(i18n
                        .get_keys()
                        .words()
                        .image()
                        .operation_canceled()
                        .inner()
                        .to_string());
                }

                let wrapped = Rc::new(new_model);

                CACHED_MODEL.with(|cached| {
                    *cached.borrow_mut() = Some(wrapped.clone());
                });

                debug!("OCR model loaded and cached");
                Ok(wrapped)
            }
            .await;

            MODEL_LOADING.with(|loading| loading.set(false));

            result?
        },
    };

    if loading_state.cancel_requested.get_untracked() {
        return Err(i18n
            .get_keys()
            .words()
            .image()
            .operation_canceled()
            .inner()
            .to_string());
    }

    loading_state.stage.set(OcrLoadingStage::Recognizing);

    info!("Running OCR with layout analysis");
    let use_case = ExtractTextFromImageUseCase::new();

    let result = execute_ocr(&use_case, model.clone(), &bytes).await;

    if loading_state.cancel_requested.get_untracked() {
        return Err(i18n
            .get_keys()
            .words()
            .image()
            .operation_canceled()
            .inner()
            .to_string());
    }

    result
}
