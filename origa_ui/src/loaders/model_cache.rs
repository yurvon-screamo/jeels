use js_sys::{Reflect, Uint8Array};
use origa::domain::OrigaError;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Cache, ReadableStream, ReadableStreamDefaultReader, Request, RequestInit, RequestMode, Response,
};

pub type ProgressCallback = Rc<dyn Fn(&str, u64, u64)>;

pub struct ModelCache {
    cache_name: String,
    on_progress: Option<ProgressCallback>,
    to_error: fn(String) -> OrigaError,
}

impl ModelCache {
    pub fn new(cache_name: impl Into<String>, to_error: fn(String) -> OrigaError) -> Self {
        Self {
            cache_name: cache_name.into(),
            on_progress: None,
            to_error,
        }
    }

    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.on_progress = Some(callback);
        self
    }

    pub async fn get_cache(&self) -> Result<Cache, OrigaError> {
        let window = web_sys::window()
            .ok_or_else(|| (self.to_error)("No window object available".to_string()))?;

        let cache_name = sanitize_cache_name(&self.cache_name);

        let cache_storage = window
            .caches()
            .map_err(|e| self.js_err("Failed to get cache storage", &e))?;

        let cache = JsFuture::from(cache_storage.open(&cache_name))
            .await
            .map_err(|e| self.js_err("Failed to open cache", &e))?;

        cache
            .dyn_into::<Cache>()
            .map_err(|e| self.js_err("Failed to cast to Cache", &e))
    }

    pub async fn ensure_files_cached(
        &self,
        cache: &Cache,
        files: &[(&str, String)],
    ) -> Result<bool, OrigaError> {
        for (filename, url) in files {
            if !self.is_file_cached(cache, filename, url).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn load_files_from_cache(
        &self,
        cache: &Cache,
        files: &[(&str, String)],
    ) -> Result<Vec<Vec<u8>>, OrigaError> {
        let mut result = Vec::with_capacity(files.len());
        for (filename, url) in files {
            let data = self.load_file_from_cache(cache, filename, url).await?;
            result.push(data);
        }
        Ok(result)
    }

    pub async fn download_and_cache_model(
        &self,
        cache: &Cache,
        files: &[(&str, String)],
    ) -> Result<Vec<Vec<u8>>, OrigaError> {
        for (filename, url) in files {
            self.download_and_cache_file(cache, filename, url).await?;
        }
        self.load_files_from_cache(cache, files).await
    }

    pub async fn is_file_cached(
        &self,
        cache: &Cache,
        filename: &str,
        url: &str,
    ) -> Result<bool, OrigaError> {
        debug!("Checking if {} is cached", filename);
        let request =
            Request::new_with_str(url).map_err(|e| self.js_err("Failed to create request", &e))?;

        let has_response = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| self.js_err("Failed to check cache", &e))?;

        Ok(!has_response.is_null() && !has_response.is_undefined())
    }

    pub async fn load_file_from_cache(
        &self,
        cache: &Cache,
        filename: &str,
        url: &str,
    ) -> Result<Vec<u8>, OrigaError> {
        let request =
            Request::new_with_str(url).map_err(|e| self.js_err("Failed to create request", &e))?;

        let response_js = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| self.js_err("Failed to get from cache", &e))?;

        if response_js.is_null() || response_js.is_undefined() {
            return Err((self.to_error)(format!("File {} not in cache", filename)));
        }

        let response = response_js
            .dyn_into::<Response>()
            .map_err(|e| self.js_err("Failed to cast to Response", &e))?;

        let array_buffer = JsFuture::from(
            response
                .array_buffer()
                .map_err(|e| self.js_err("Failed to get array buffer", &e))?,
        )
        .await
        .map_err(|e| self.js_err("Failed to read array buffer", &e))?;

        let uint8_array = Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    pub async fn download_and_cache_file(
        &self,
        cache: &Cache,
        filename: &str,
        url: &str,
    ) -> Result<Vec<u8>, OrigaError> {
        info!("Downloading {} from {}", filename, url);

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| self.js_err("Failed to create request", &e))?;

        let window = web_sys::window()
            .ok_or_else(|| (self.to_error)("No window object available".to_string()))?;

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| self.js_err(format!("Failed to fetch {}", filename), &e))?
            .dyn_into::<Response>()
            .map_err(|e| self.js_err("Failed to cast to Response", &e))?;

        if !response.ok() {
            return Err((self.to_error)(format!(
                "Failed to download {}: HTTP {}",
                filename,
                response.status()
            )));
        }

        let total_bytes = get_content_length(&response);

        let stream = response
            .body()
            .ok_or_else(|| (self.to_error)("Response body is not a stream".to_string()))?;

        let reader = get_stream_reader(&stream, self.to_error)?;

        let data = self
            .read_stream_with_progress(&reader, filename, total_bytes)
            .await?;

        debug!("Downloaded {} bytes for {}", data.len(), filename);

        let mut data_for_cache = data.clone();
        let cache_request = Request::new_with_str(url)
            .map_err(|e| self.js_err("Failed to create cache request", &e))?;

        let cache_response_init = web_sys::ResponseInit::new();
        let cache_response = Response::new_with_opt_u8_array_and_init(
            Some(&mut data_for_cache[..]),
            &cache_response_init,
        )
        .map_err(|e| self.js_err("Failed to create cache response", &e))?;

        debug!("Caching {}", filename);
        JsFuture::from(cache.put_with_request(&cache_request, &cache_response))
            .await
            .map_err(|e| self.js_err(format!("Failed to cache {}", filename), &e))?;

        Ok(data)
    }

    async fn read_stream_with_progress(
        &self,
        reader: &ReadableStreamDefaultReader,
        filename: &str,
        total_bytes: Option<u64>,
    ) -> Result<Vec<u8>, OrigaError> {
        let chunks: Rc<RefCell<Vec<Vec<u8>>>> = Rc::new(RefCell::new(Vec::new()));
        let loaded: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

        loop {
            let read_result = JsFuture::from(reader.read())
                .await
                .map_err(|e| self.js_err("Failed to read from stream", &e))?;

            let done = Reflect::get(&read_result, &JsValue::from_str("done"))
                .map_err(|e| self.js_err("Failed to get done flag", &e))?
                .as_bool()
                .unwrap_or(true);

            if done {
                break;
            }

            let value = Reflect::get(&read_result, &JsValue::from_str("value"))
                .map_err(|e| self.js_err("Failed to get chunk value", &e))?;

            let chunk = Uint8Array::new(&value);
            let chunk_len = chunk.length() as u64;
            let chunk_vec = chunk.to_vec();

            chunks.borrow_mut().push(chunk_vec);

            let mut loaded_guard = loaded.borrow_mut();
            *loaded_guard += chunk_len;

            if let Some(ref callback) = self.on_progress {
                callback(filename, *loaded_guard, total_bytes.unwrap_or(0));
            }
        }

        let chunks_guard = chunks.borrow();
        let total_size: usize = chunks_guard.iter().map(|c| c.len()).sum();
        let mut result = Vec::with_capacity(total_size);
        for chunk in chunks_guard.iter() {
            result.extend_from_slice(chunk);
        }

        Ok(result)
    }

    fn js_err(&self, msg: impl AsRef<str>, e: &JsValue) -> OrigaError {
        (self.to_error)(format!("{}: {:?}", msg.as_ref(), e))
    }
}

fn sanitize_cache_name(name: &str) -> String {
    const MAX_CACHE_NAME_LEN: usize = 64;

    let sanitized: String = name
        .chars()
        .take(MAX_CACHE_NAME_LEN)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let sanitized = sanitized.replace("..", "_");

    if sanitized.is_empty() {
        "default_cache".to_string()
    } else {
        sanitized
    }
}

fn get_content_length(response: &Response) -> Option<u64> {
    response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u64>().ok())
}

fn get_stream_reader(
    stream: &ReadableStream,
    to_error: fn(String) -> OrigaError,
) -> Result<ReadableStreamDefaultReader, OrigaError> {
    stream
        .get_reader()
        .dyn_into::<ReadableStreamDefaultReader>()
        .map_err(|e| {
            to_error(format!(
                "Failed to cast to ReadableStreamDefaultReader: {:?}",
                e
            ))
        })
}
