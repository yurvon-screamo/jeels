use crate::ui_components::{OcrLoadingStage, OcrLoadingState};
use leptos::prelude::Set;
use origa::ocr::JapaneseOCRModel;
use origa::use_cases::ExtractTextFromImageUseCase;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::File;
use web_sys::js_sys::Function;

pub(crate) const MAX_BASE64_LEN: usize = 15_000_000;

pub(crate) fn is_image_file(file: &File) -> bool {
    let mime = file.type_();
    mime == "image/png" || mime == "image/jpeg" || mime == "image/webp" || mime.is_empty()
}

pub(crate) async fn read_file_as_data_url(file: &File) -> Result<String, String> {
    let reader =
        web_sys::FileReader::new().map_err(|e| format!("Failed to create FileReader: {:?}", e))?;

    let reader_clone = reader.clone();
    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Rc::new(RefCell::new(Some(sender)));
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        if let Some(sender) = sender.borrow_mut().take() {
            let _ = sender.send(());
        }
    });

    if let Some(func) = closure.as_ref().dyn_ref::<Function>() {
        reader.set_onloadend(Some(func));
    }
    closure.forget();

    reader
        .read_as_data_url(file)
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    receiver
        .await
        .map_err(|_| "Failed to wait for file read".to_string())?;

    let result = reader_clone
        .result()
        .map_err(|e| format!("Failed to get result: {:?}", e))?;
    result
        .as_string()
        .ok_or_else(|| "Result is not a string".to_string())
}

pub(crate) fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD};

    if input.len() > MAX_BASE64_LEN {
        return Err(format!(
            "Input too large: {} bytes (max {})",
            input.len(),
            MAX_BASE64_LEN
        ));
    }

    let input = input.replace(['\n', '\r'], "");

    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return Err("Invalid base64 characters".to_string());
    }

    let padding = (4 - input.len() % 4) % 4;
    let padded_input = format!("{}{}", input, "=".repeat(padding));

    STANDARD
        .decode(&padded_input)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

pub(crate) fn calculate_speed_and_eta(
    start_time: Option<f64>,
    loaded: u64,
    total: u64,
) -> (u64, u64) {
    let Some(start) = start_time else {
        return (0, 0);
    };

    let elapsed_ms = web_sys::js_sys::Date::now() - start;
    if elapsed_ms <= 0.0 {
        return (0, 0);
    }

    let elapsed_secs = elapsed_ms / 1000.0;
    let speed_bps = (loaded as f64 / elapsed_secs) as u64;

    if speed_bps == 0 || total == 0 || loaded >= total {
        return (speed_bps, 0);
    }

    let remaining_bytes = total - loaded;
    let eta_seconds = (remaining_bytes as f64 / speed_bps as f64) as u64;

    (speed_bps, eta_seconds)
}

pub(super) async fn init_ocr_model(
    model_files: origa::ocr::ModelFiles,
    loading_state: &OcrLoadingState,
) -> Result<JapaneseOCRModel, String> {
    #[cfg(target_arch = "wasm32")]
    {
        JapaneseOCRModel::from_model_files(model_files)
            .await
            .map_err(|e| {
                loading_state.stage.set(OcrLoadingStage::Error {
                    stage: "init".to_string(),
                    message: format!("Failed to initialize OCR model: {:?}", e),
                });
                format!("Failed to initialize OCR model: {:?}", e)
            })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        JapaneseOCRModel::from_model_files(model_files).map_err(|e| {
            loading_state.stage.set(OcrLoadingStage::Error {
                stage: "init".to_string(),
                message: format!("Failed to initialize OCR model: {:?}", e),
            });
            format!("Failed to initialize OCR model: {:?}", e)
        })
    }
}

pub(super) async fn execute_ocr(
    use_case: &ExtractTextFromImageUseCase,
    model: Rc<JapaneseOCRModel>,
    bytes: &[u8],
) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use_case
            .execute(model.clone(), bytes)
            .await
            .map_err(|e| format!("OCR failed: {:?}", e))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use_case
            .execute(model.clone(), bytes)
            .map_err(|e| format!("OCR failed: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decode_roundtrips_ascii() {
        // "hello" -> "aGVsbG8="
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello".to_vec());
    }

    #[test]
    fn base64_decode_invalid_input_is_error() {
        assert!(base64_decode("!!!not-base64!!!").is_err());
    }

    #[test]
    fn speed_and_eta_without_start_time_is_zero() {
        assert_eq!(calculate_speed_and_eta(None, 100, 1000), (0, 0));
    }

    // NOTE: the "completed transfer → ETA 0" case needs a real clock
    // (`js_sys::Date::now` is wasm-only); it is exercised implicitly by the
    // WASM OCR-stage tests.
}
