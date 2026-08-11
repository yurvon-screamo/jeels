//! Build tool: generates pre-built rkyv lindera dictionary blob for CDN.
//!
//! Reads the 8 compressed lindera files from `cdn/dictionaries/`,
//! decompresses them, builds lindera structures, serializes to rkyv,
//! and writes the result to `cdn/dictionaries/cached-lindera.bin`.
//!
//! The output file is uploaded to CDN and downloaded directly by the
//! WASM client — no build/serialize happens on-device.
//!
//! Usage:
//!     cargo run --bin build-lindera-cache

use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::DeflateDecoder;

use origa::domain::{
    CachedLinderaDictionary, DictionaryData, build_cached_lindera, serialize_cached_lindera_to_rkyv,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR must be set (run via `cargo run --bin`)")?;

    let cdn_dir = Path::new(&manifest_dir)
        .parent()
        .ok_or("Failed to find workspace root")?
        .join("cdn")
        .join("dictionaries");

    println!("Reading lindera files from {}...", cdn_dir.display());

    let data = load_and_decompress(&cdn_dir)?;
    println!("DictionaryData loaded, building lindera structures...");

    let cached = build_cached_lindera(data)
        .map_err(|e| format!("Failed to build lindera structures: {e}"))?;
    println!("Lindera structures built successfully.");

    println!("Serializing to rkyv...");
    let bytes = serialize_cached_lindera_to_rkyv(&cached)
        .map_err(|e| format!("Failed to serialize: {e}"))?;

    let output_path = cdn_dir.join("cached-lindera.bin");
    let size_mb = bytes.len() as f64 / 1_048_576.0;
    println!(
        "Writing rkyv blob ({} bytes, {:.1} MB) to {}...",
        bytes.len(),
        size_mb,
        output_path.display()
    );

    fs::write(&output_path, &bytes)
        .map_err(|e| format!("Failed to write {}: {e}", output_path.display()))?;

    // Verify: round-trip
    println!("Verifying round-trip...");
    let _: CachedLinderaDictionary =
        rkyv::from_bytes::<CachedLinderaDictionary, rkyv::rancor::Error>(&bytes)
            .map_err(|e| format!("Round-trip verification failed: {e}"))?;
    println!("Round-trip OK. Done.");

    Ok(())
}

fn load_and_decompress(dict_dir: &Path) -> Result<DictionaryData, String> {
    let read_file = |name: &str| -> Result<Vec<u8>, String> {
        let path = dict_dir.join(name);
        fs::read(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))
    };

    let decompress = |name: &str, data: Vec<u8>| -> Result<Vec<u8>, String> {
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| format!("Failed to decompress {name}: {e}"))?;
        Ok(decompressed)
    };

    Ok(DictionaryData {
        char_def: decompress("char_def.bin", read_file("char_def.bin")?)?,
        matrix: decompress("matrix.mtx", read_file("matrix.mtx")?)?,
        dict_da: decompress("dict.da", read_file("dict.da")?)?,
        dict_vals: decompress("dict.vals", read_file("dict.vals")?)?,
        unk: decompress("unk.bin", read_file("unk.bin")?)?,
        words_idx: decompress("dict.wordsidx", read_file("dict.wordsidx")?)?,
        words: decompress("dict.words", read_file("dict.words")?)?,
        metadata: read_file("metadata.json")?,
    })
}
