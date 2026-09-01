use flate2::read::DeflateDecoder;
use origa::domain::{DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

fn decompress(data: Vec<u8>) -> Result<Vec<u8>, OrigaError> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Decompression failed: {}", e),
        })?;
    Ok(decompressed)
}

fn cache_dir() -> PathBuf {
    PathBuf::from("target/cdn-cache")
}

pub fn load_dictionary() -> Result<(), OrigaError> {
    load_dictionary_from(None)
}

pub fn load_dictionary_from(dictionary_dir: Option<&Path>) -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        return Ok(());
    }

    let dict_dir = match dictionary_dir {
        Some(path) if path.exists() => {
            tracing::info!(
                "using dictionary dir from --dictionary-dir: {}",
                path.display()
            );
            path.to_path_buf()
        },
        Some(path) => {
            tracing::warn!(
                "--dictionary-dir '{}' does not exist, falling back to default lookup",
                path.display()
            );
            find_dictionary_directory()?
        },
        None => find_dictionary_directory()?,
    };

    let read_file = |name: &str| -> Result<Vec<u8>, OrigaError> {
        fs::read(dict_dir.join(name)).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", name, e),
        })
    };

    let data = DictionaryData {
        char_def: decompress(read_file("char_def.bin")?)?,
        matrix: decompress(read_file("matrix.mtx")?)?,
        dict_trie: decompress(read_file("dict.trie")?)?,
        dict_vals_idx: decompress(read_file("dict.valsidx")?)?,
        dict_vals: decompress(read_file("dict.vals")?)?,
        unk: decompress(read_file("unk.bin")?)?,
        words_idx: decompress(read_file("dict.wordsidx")?)?,
        words: decompress(read_file("dict.words")?)?,
        metadata: read_file("metadata.json")?,
    };

    init_dictionary(data)
}

fn find_dictionary_directory() -> Result<PathBuf, OrigaError> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let mut dict_dir = PathBuf::from(manifest_dir);

    if dict_dir.ends_with("tokenizer") {
        dict_dir.pop();
    }

    let dict_dir = dict_dir
        .join("cdn")
        .join("dictionaries")
        .join(origa::domain::SUDACHIDICT_DIR);

    if dict_dir.exists() {
        return Ok(dict_dir);
    }
    let cache = cache_dir();
    if cache.exists() {
        tracing::info!("using cached CDN dictionary from {}", cache.display());
        return Ok(cache);
    }

    tracing::error!("local dictionary not found, you need to download from CDN");
    Ok(cache)
}
