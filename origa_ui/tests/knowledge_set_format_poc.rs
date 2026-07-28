//! Slice 1 — blocking PoC-gate: choose the wire format for `knowledge_set`
//! by measurement, not by postulate.
//!
//! Compares four candidate formats on a representative fixture:
//!   1. raw JSON (current wire format, the baseline we must beat)
//!   2. bincode-only (compact binary, zero compression CPU cost)
//!   3. deflate(JSON) — gzip-style compression over JSON text
//!   4. deflate(bincode) — compression over compact binary
//!
//! Each compressed variant is measured as the FINAL wire string
//! (magic-prefix + base64), because the TrailBase column is `TEXT`
//! (`Option<String>`), so raw deflate bytes cannot be stored directly and
//! base64's ~33% overhead is unavoidable. The gate compares the real
//! on-the-wire length.
//!
//! Critical check: `KnowledgeSet` carries `#[serde(flatten)] stats` and a
//! custom `deserialize_study_cards`. bincode 1.x has a known incompatibility
//! with `serde(flatten)` (flatten needs self-describing `deserialize_any`).
//! This PoC explicitly tests bincode roundtrip and records failure rather
//! than assuming compatibility — a failing roundtrip disqualifies the
//! bincode variants regardless of size.
//!
//! Format choice rule (per the approved plan v3):
//!
//! - latency budget: encode + decode on the fixture <= 500 ms
//!   (sync-checkpoint path, not the rating hot-path)
//! - pick the variant with the smallest wire string that passes the budget
//! - bincode variants are valid only if their roundtrip succeeds; a
//!   serde(flatten) incompatibility disqualifies them regardless of size
//!
//! Outcome (release, ~8 MiB fixture): bincode disqualified by roundtrip
//! failure; deflate(JSON) level 6 chosen — 4.69x ratio at ~197ms
//! encode+decode. See `knowledge_set_codec::DEFLATE_LEVEL`.
//!
//! This is a `#[test]`; run with `cargo test -p origa_ui --test
//! knowledge_set_format_poc -- --nocapture --ignored` to read the table.

use std::io::Read;
use std::io::Write;
use std::time::Instant;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use origa::domain::{Card, KnowledgeSet, NativeLanguage, PhraseCard, RateMode, Rating, User};
use ulid::Ulid;

const MAGIC_PREFIX: &str = "DEFLATE;";
const N_CARDS: usize = 6000;
const REVIEWS_PER_CARD: usize = 8;

fn build_fixture(n_cards: usize, reviews_per_card: usize) -> User {
    let mut user = User::new("poc@test".to_string(), NativeLanguage::Russian, None);
    let ratings = [Rating::Easy, Rating::Good, Rating::Hard, Rating::Again];
    for _ in 0..n_cards {
        let card = Card::Phrase(PhraseCard::new(Ulid::new()));
        let study_card = match user.create_card(card) {
            Ok(sc) => sc,
            Err(e) => panic!("create_card failed: {e:?}"),
        };
        let card_id = *study_card.card_id();
        for r in 0..reviews_per_card {
            let rating = ratings[r % ratings.len()];
            user.rate_card(card_id, rating, RateMode::StandardLesson)
                .expect("rate_card");
        }
    }
    user
}

fn deflate_encode(data: &[u8], level: u32) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data).expect("deflate write");
    encoder.finish().expect("deflate finish")
}

fn deflate_decode(data: &[u8]) -> Vec<u8> {
    let mut decoder = DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).expect("deflate decode");
    out
}

struct VariantResult {
    name: &'static str,
    wire_len: usize,
    encode_us: u128,
    decode_us: u128,
    roundtrip_ok: bool,
    note: &'static str,
}

impl VariantResult {
    fn print_row(&self, baseline_json_len: usize) {
        let ratio = if baseline_json_len == 0 {
            0.0
        } else {
            baseline_json_len as f64 / self.wire_len as f64
        };
        println!(
            "{:<18} wire={:>8}B  ratio={:>5.2}x  enc={:>6}us  dec={:>6}us  roundtrip={}  {}",
            self.name,
            self.wire_len,
            ratio,
            self.encode_us,
            self.decode_us,
            if self.roundtrip_ok { "OK" } else { "FAIL" },
            self.note,
        );
    }
}

#[test]
#[ignore = "PoC-gate: run explicitly with --ignored to read the decision table"]
fn compare_knowledge_set_wire_formats() {
    let user = build_fixture(N_CARDS, REVIEWS_PER_CARD);
    let ks: &KnowledgeSet = user.knowledge_set();

    // Baseline: current wire format — plain JSON string in a TEXT column.
    let json_encode_start = Instant::now();
    let json_string = serde_json::to_string(ks).expect("serde_json encode");
    let json_encode_us = json_encode_start.elapsed().as_micros();
    let json_bytes = json_string.into_bytes();

    let json_decode_start = Instant::now();
    let _json_roundtrip: KnowledgeSet =
        serde_json::from_slice(&json_bytes).expect("serde_json decode");
    let json_decode_us = json_decode_start.elapsed().as_micros();

    let baseline_json_len = json_bytes.len();
    let raw_json = VariantResult {
        name: "raw-JSON",
        wire_len: baseline_json_len,
        encode_us: json_encode_us,
        decode_us: json_decode_us,
        roundtrip_ok: true,
        note: "current baseline",
    };

    println!();
    println!(
        "fixture: {N_CARDS} cards × {REVIEWS_PER_CARD} reviews; raw JSON = {baseline_json_len} bytes ({:.2} MiB)",
        baseline_json_len as f64 / (1024.0 * 1024.0)
    );
    println!("------------------------------------------------------------------");
    raw_json.print_row(baseline_json_len);

    // Variant 2: bincode-only. Note: TEXT column forces base64 even here.
    let bincode_result = match bincode::serialize(ks) {
        Ok(bincode_bytes) => {
            let bincode_roundtrip_ok = bincode::deserialize::<KnowledgeSet>(&bincode_bytes).is_ok();

            let b64 = BASE64.encode(&bincode_bytes);
            let wire = format!("BINCODE;{b64}");

            // decode latency only meaningful if roundtrip is sound
            let decode_us = if bincode_roundtrip_ok {
                let start = Instant::now();
                let raw = BASE64
                    .decode(wire.strip_prefix("BINCODE;").unwrap())
                    .expect("b64 decode");
                let _ks: KnowledgeSet = bincode::deserialize(&raw).expect("bincode decode");
                start.elapsed().as_micros()
            } else {
                0
            };

            VariantResult {
                name: "bincode+base64",
                wire_len: wire.len(),
                encode_us: 0,
                decode_us,
                roundtrip_ok: bincode_roundtrip_ok,
                note: if bincode_roundtrip_ok {
                    ""
                } else {
                    "DISQUALIFIED: roundtrip failed (serde(flatten) incompatible)"
                },
            }
        },
        Err(_e) => VariantResult {
            name: "bincode+base64",
            wire_len: 0,
            encode_us: 0,
            decode_us: 0,
            roundtrip_ok: false,
            note: "DISQUALIFIED: serialize failed (serde(flatten) incompatible)",
        },
    };
    bincode_result.print_row(baseline_json_len);

    // Variants 3 & 4: deflate over JSON and over bincode, levels 1 / 6 / 9.
    for level in [1u32, 6, 9] {
        // deflate(JSON) + base64 + magic prefix
        let enc_start = Instant::now();
        let deflated_json = deflate_encode(&json_bytes, level);
        let wire_json = format!("{MAGIC_PREFIX}{}", BASE64.encode(&deflated_json));
        let enc_us = enc_start.elapsed().as_micros();

        let dec_start = Instant::now();
        let raw = BASE64
            .decode(wire_json.strip_prefix(MAGIC_PREFIX).unwrap())
            .expect("b64 decode");
        let inflated = deflate_decode(&raw);
        let _ks: KnowledgeSet =
            serde_json::from_slice(&inflated).expect("json decode after deflate");
        let dec_us = dec_start.elapsed().as_micros();

        VariantResult {
            name: "deflate(JSON)",
            wire_len: wire_json.len(),
            encode_us: enc_us,
            decode_us: dec_us,
            roundtrip_ok: true,
            note: "",
        }
        .print_row(baseline_json_len);

        // Only test deflate(bincode) if bincode roundtrip itself is sound;
        // otherwise compressing an un-decodable payload is pointless.
        if bincode_result.roundtrip_ok {
            if let Ok(bincode_bytes) = bincode::serialize(ks) {
                let enc_start = Instant::now();
                let deflated_bincode = deflate_encode(&bincode_bytes, level);
                let wire_bincode = format!("DEFBINCODE;{}", BASE64.encode(&deflated_bincode));
                let enc_us = enc_start.elapsed().as_micros();

                let dec_start = Instant::now();
                let raw = BASE64
                    .decode(wire_bincode.strip_prefix("DEFBINCODE;").unwrap())
                    .expect("b64 decode");
                let inflated = deflate_decode(&raw);
                let _ks: KnowledgeSet =
                    bincode::deserialize(&inflated).expect("bincode decode after deflate");
                let dec_us = dec_start.elapsed().as_micros();

                VariantResult {
                    name: "deflate(bincode)",
                    wire_len: wire_bincode.len(),
                    encode_us: enc_us,
                    decode_us: dec_us,
                    roundtrip_ok: true,
                    note: "",
                }
                .print_row(baseline_json_len);
            }
        }
    }
    println!("------------------------------------------------------------------");
    println!("latency budget: encode+decode <= 500ms (sync checkpoint path)");
    println!(
        "decision: smallest wire_len passing budget; bincode variants valid only if roundtrip OK"
    );
}
