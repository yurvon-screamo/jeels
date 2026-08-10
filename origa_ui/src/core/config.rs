use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub fn public_url(path: &str) -> String {
    if !path.starts_with('/') {
        tracing::warn!(
            "public_url called with relative path '{}', expected absolute path starting with '/'",
            path
        );
    }

    let base = option_env!("ORIGA_PUBLIC_BASE_URL").unwrap_or("");
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), path)
    }
}

static SIGNING_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_current_date() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new_0();
        format!(
            "{:04}{:02}{:02}",
            d.get_full_year(),
            d.get_month() + 1,
            d.get_date()
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().format("%Y%m%d").to_string()
    }
}

pub fn cdn_url(path: &str) -> String {
    let base = env!("ORIGA_CDN_BASE_URL").trim_end_matches('/');
    let date = get_current_date();

    let (clean_path, _user_query) = match path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (path, None),
    };

    let cache = SIGNING_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_key = format!("{date}:{path}");

    {
        let guard = cache.lock().expect("signing cache lock");
        if let Some(cached) = guard.get(&cache_key) {
            return cached.clone();
        }
    }

    let url = format!("{}{}", base, clean_path,);

    {
        let mut guard = cache.lock().expect("signing cache lock");
        guard.insert(cache_key, url.clone());
    }

    url
}

#[cfg(target_arch = "wasm32")]
pub fn whisper_base_url() -> String {
    cdn_url("/whisper")
}
