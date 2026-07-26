#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub ndlocr_base_url: String,
    pub ndlocr_cache_dir: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            ndlocr_base_url: String::new(),
            ndlocr_cache_dir: "ndlocr-model-".to_string(),
        }
    }
}

impl ModelConfig {
    pub fn new(ndlocr_base_url: impl Into<String>, ndlocr_cache_dir: impl Into<String>) -> Self {
        Self {
            ndlocr_base_url: ndlocr_base_url.into(),
            ndlocr_cache_dir: ndlocr_cache_dir.into(),
        }
    }

    pub fn ndlocr_file_names() -> &'static [&'static str] {
        &[
            "deim.onnx",
            "parseq-30.onnx",
            "parseq-50.onnx",
            "parseq-100.onnx",
            "vocab.txt",
        ]
    }

    pub fn model_url(&self, filename: &str) -> String {
        format!(
            "{}/{}",
            self.ndlocr_base_url.trim_end_matches('/'),
            filename.trim_start_matches('/')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ModelConfig::new tests
    #[test]
    fn model_config_new() {
        let config = ModelConfig::new("https://example.com", "cache");
        assert_eq!(config.ndlocr_base_url, "https://example.com");
        assert_eq!(config.ndlocr_cache_dir, "cache");
    }

    // ModelConfig::default tests
    #[test]
    fn model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.ndlocr_base_url, "");
        assert_eq!(config.ndlocr_cache_dir, "ndlocr-model-");
    }

    // model_url tests
    #[test]
    fn model_url_normal() {
        let config = ModelConfig::new("https://example.com", "");
        assert_eq!(config.model_url("file.txt"), "https://example.com/file.txt");
    }

    #[test]
    fn model_url_trims_trailing_slash_from_base() {
        let config = ModelConfig::new("https://example.com/", "");
        assert_eq!(config.model_url("file.txt"), "https://example.com/file.txt");
    }

    #[test]
    fn model_url_trims_leading_slash_from_filename() {
        let config = ModelConfig::new("https://example.com", "");
        assert_eq!(
            config.model_url("/file.txt"),
            "https://example.com/file.txt"
        );
    }

    #[test]
    fn model_url_trims_both() {
        let config = ModelConfig::new("https://example.com/", "");
        assert_eq!(
            config.model_url("/file.txt"),
            "https://example.com/file.txt"
        );
    }

    #[test]
    fn model_url_empty_base() {
        let config = ModelConfig::new("", "");
        assert_eq!(config.model_url("file.txt"), "/file.txt");
    }

    #[test]
    fn model_url_complex_filename() {
        let config = ModelConfig::new("https://example.com", "");
        assert_eq!(
            config.model_url("path/to/model.onnx"),
            "https://example.com/path/to/model.onnx"
        );
    }

    // ndlocr_file_names tests
    #[test]
    fn ndlocr_file_names_returns_expected_files() {
        let files = ModelConfig::ndlocr_file_names();

        assert_eq!(files.len(), 5);
        assert!(files.contains(&"deim.onnx"));
        assert!(files.contains(&"parseq-30.onnx"));
        assert!(files.contains(&"parseq-50.onnx"));
        assert!(files.contains(&"parseq-100.onnx"));
        assert!(files.contains(&"vocab.txt"));
    }
}
