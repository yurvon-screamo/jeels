use origa::domain::{DictionaryData, init_dictionary, is_dictionary_loaded, tokenize_text};
use std::fs;
use std::io::Read;

fn decompress(data: Vec<u8>) -> Vec<u8> {
    let mut decoder = flate2::read::DeflateDecoder::new(&data[..]);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).unwrap();
    out
}

fn main() {
    if !is_dictionary_loaded() {
        let dir = std::path::Path::new("cdn/dictionaries");
        let rf = |n: &str| fs::read(dir.join(n)).unwrap();
        let data = DictionaryData {
            char_def: decompress(rf("char_def.bin")),
            matrix: decompress(rf("matrix.mtx")),
            dict_trie: decompress(rf("dict.trie")),
            dict_vals_idx: decompress(rf("dict.valsidx")),
            dict_vals: decompress(rf("dict.vals")),
            unk: decompress(rf("unk.bin")),
            words_idx: decompress(rf("dict.wordsidx")),
            words: decompress(rf("dict.words")),
            metadata: rf("metadata.json"),
        };
        init_dictionary(data).unwrap();
    }
    for text in [
        "どういう",
        "こういう",
        "そういう",
        "カッコイイ！",
        "かっこいい",
        "カッコイイですね",
    ] {
        let toks = tokenize_text(text).unwrap();
        println!("=== {text} ===");
        for t in &toks {
            println!(
                "  surface={} | base={} | pos={:?} | vocab={}",
                t.orthographic_surface_form(),
                t.orthographic_base_form(),
                t.part_of_speech(),
                t.part_of_speech().is_vocabulary_word()
            );
        }
        // raw: повторная сегментация с дампом деталей
        if let Some(seg) = origa::domain::tokenizer::segmenter_pub() {
            let mut toks2 = seg.segment(text.into()).unwrap();
            for tok in toks2.iter_mut() {
                let pos = tok.get("part_of_speech").unwrap_or("?").to_string();
                let sub = tok
                    .get("part_of_speech_subcategory_1")
                    .unwrap_or("?")
                    .to_string();
                let nf = tok.get("normalized_form").unwrap_or("?").to_string();
                let det = tok.details().join("|");
                println!(
                    "  RAW surface={} id={} unk={} sys={} pos={} sub={} nf={} details=[{}]",
                    tok.surface,
                    tok.word_id.id(),
                    tok.word_id.is_unknown(),
                    tok.word_id.is_system(),
                    pos,
                    sub,
                    nf,
                    det
                );
            }
        }
    }
}
