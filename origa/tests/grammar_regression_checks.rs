//! Regression tests for grammar rules that were fixed during the N3+N2 work.
//!
//! Each test verifies that a specific ``rule_id`` (loaded from
//! ``cdn/grammar/grammar_v2.json``) produces the correct Japanese form for the
//! test verbs. These tests are NOT ``#[ignore]`` — they run as part of the
//! normal suite so a future contributor who breaks a format_map chain sees
//! the regression immediately.
//!
//! The ``cdn/`` directory is gitignored (cannot be restored from version
//! control). On a fresh clone without the grammar store the tests
//! **gracefully skip** (pass with a stderr note) rather than panic, so
//! ``cargo test --workspace`` stays green in CI environments that do not
//! have access to the CDN artifacts. Once the store is present (local dev,
//! release CI), every regression check runs for real.
//!
//! Run: ``cargo test -p origa --test grammar_regression_checks``.

use std::path::PathBuf;
use std::sync::Once;

use origa::dictionary::grammar::{GrammarData, get_rule_by_id, init_grammar, is_grammar_loaded};
use origa::domain::PartOfSpeech;
use ulid::Ulid;

// Parallel tests share the process-wide ``GRAMMAR_RULES`` OnceLock, so the
// load must run exactly once per test binary. ``Once::call_once`` serialises
// the first callers and lets the rest observe the already-loaded state,
// avoiding the "Failed to set grammar rules" panic that ``init_grammar``
// returns when a second thread races the ``set`` after the first succeeded.
static GRAMMAR_INIT: Once = Once::new();

fn cdn_grammar_path() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of the origa crate manifest")
        .join("cdn")
        .join("grammar")
        .join("grammar_v2.json")
}

/// Try once to load the grammar store. Returns ``true`` when the store is
/// available and loaded (either by this call or a prior one), ``false`` when
/// the gitignored ``cdn/grammar/grammar_v2.json`` is absent so the caller can
/// skip gracefully. A malformed store still panics — a corrupt grammar_v2.json
/// is a real error, not a missing-environment condition.
fn ensure_grammar_loaded() -> bool {
    GRAMMAR_INIT.call_once(|| {
        let path = cdn_grammar_path();
        if !path.exists() {
            // Leave the OnceLock unset; is_grammar_loaded() stays false and
            // callers skip gracefully rather than panic.
            return;
        }
        let grammar_json = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!(
                "cannot read grammar store at {}: {} — check file permissions \
                 or restore cdn/grammar/grammar_v2.json via scripts/deploy_cdn.py",
                path.display(),
                err
            )
        });
        init_grammar(GrammarData { grammar_json }).expect("init_grammar must succeed");
    });
    is_grammar_loaded()
}

/// Apply the format_map chain of ``rule_id_str`` for ``pos`` to each
/// ``(verb, expected)`` case, asserting equality with a message that names
/// the rule, the verb, and both forms so a failure is self-explanatory.
///
/// Returns early (no-op pass) when the grammar store is absent, so the
/// workspace test suite stays green on fresh clones without the gitignored
/// ``cdn/`` artifacts.
fn assert_rule_formats(rule_id_str: &str, pos: PartOfSpeech, cases: &[(&str, &str)]) {
    if !ensure_grammar_loaded() {
        eprintln!(
            "skip grammar regression check for {rule_id_str}: \
             cdn/grammar/grammar_v2.json absent (cdn/ is gitignored; \
             restore via scripts/deploy_cdn.py)"
        );
        return;
    }
    let rule_id = Ulid::from_string(rule_id_str).expect("rule_id must be a valid ULID");
    let rule = get_rule_by_id(&rule_id).unwrap_or_else(|| {
        panic!("rule {rule_id_str} not found in grammar_v2.json — stale fixture?")
    });
    for (verb, expected) in cases {
        let result = rule
            .format(verb, &pos)
            .unwrap_or_else(|e| panic!("rule {rule_id_str} format({verb}) failed: {e:?}"));
        assert_eq!(
            result, *expected,
            "rule {rule_id_str} format({verb}): expected {expected:?}, got {result:?}"
        );
    }
}

// ===========================================================================
// Bug fixes (engine-level): ざる mizenkei, くる tara irregular
// ===========================================================================

/// Bug: ``ざる`` for ``する`` previously gave ``しざる`` (renyokei) instead of
/// ``せざる`` (mizenkei). The chain ``VerbToMizenkei → AddPostfix「ざる」`` is
/// the canonical ざるを得ない / ざる（打ち消し連体）construction; ``する`` and
/// ``くる`` must conjugate irregularly (せ / こ), not through the regular
/// renyokei stem (し / き).
#[test]
fn zaru_uses_mizenkei_not_renyokei_for_suru() {
    assert_rule_formats(
        "01KV2C0RKHWT3G177AW3GPKJA6",
        PartOfSpeech::Verb,
        &[
            ("書く", "書かざる"),
            ("食べる", "食べざる"),
            ("する", "せざる"), // CRITICAL: was しざる before the fix
            ("くる", "こざる"),
        ],
    );
}

/// Bug: ``VerbToTara`` for ``くる`` previously produced ``れたら`` (applying the
/// godan る rule) instead of ``きたら`` (irregular). The ``TARA_IRREGULAR``
/// constant in ``forms_verb/irregulars.rs`` must map ``くる`` → ``きたら`` so
/// every たら-format rule (conditional, hypothetical, discovery) renders correctly.
#[test]
fn tara_irregular_kuru_is_kitara() {
    assert_rule_formats(
        "01KV2BRAW30ESEMGXK3N2PTAEA",
        PartOfSpeech::Verb,
        &[
            ("書く", "書いたら"),
            ("する", "したら"),
            ("くる", "きたら"), // CRITICAL: was れたら before the fix
            ("食べる", "食べたら"),
        ],
    );
}

// ===========================================================================
// 6 N3+N2 chain fixes
// ===========================================================================

/// Bug: ``～てもらえませんか`` produced ``書いててもらえませんか`` (spurious double-て)
/// because the postfix started with ``て``. The postfix must be ``もらえませんか``
/// so ``VerbToTeForm → AddPostfix`` yields ``書いてもらえませんか``.
#[test]
fn te_moraemasenka_no_double_te() {
    assert_rule_formats(
        "01KV2BRAW30ESEMGXK3N2PTADV",
        PartOfSpeech::Verb,
        &[
            ("書く", "書いてもらえませんか"),
            ("食べる", "食べてもらえませんか"),
            ("する", "してもらえませんか"),
        ],
    );
}

/// Bug: ``～てもかまわない`` produced ``書いててもかまわない`` (double-て). The
/// postfix must be ``もかまわない`` so the chain yields ``書いてもかまわない``.
#[test]
fn te_mo_kamawanai_no_double_te() {
    assert_rule_formats(
        "01KV2BV49PQW2NZ6JX06ZRX0F5",
        PartOfSpeech::Verb,
        &[
            ("書く", "書いてもかまわない"),
            ("食べる", "食べてもかまわない"),
        ],
    );
}

/// Bug: ``～つもりはない`` produced ``書きつもりはない`` (stem form) instead of
/// ``書くつもりはない`` (dictionary form). ``つもり`` attaches to the plain
/// non-past (dictionary) form, so the format_map must be a bare
/// ``AddPostfix`` with no verb-class action in front of it.
#[test]
fn tsumori_wa_nai_uses_dictionary_form() {
    assert_rule_formats(
        "01KV2C1TJN7FZ34PB80VBFCXES",
        PartOfSpeech::Verb,
        &[
            ("書く", "書くつもりはない"), // CRITICAL: was 書きつもりはない
            ("食べる", "食べるつもりはない"),
            ("する", "するつもりはない"),
        ],
    );
}

/// Bug: ``～つもりだった`` produced ``書きつもりだった`` (stem) instead of
/// ``書くつもりだった`` (dictionary form). Same root cause as つもりはない.
#[test]
fn tsumori_datta_uses_dictionary_form() {
    assert_rule_formats(
        "01KV2C1TJN7FZ34PB80VBFCXET",
        PartOfSpeech::Verb,
        &[
            ("書く", "書くつもりだった"),
            ("食べる", "食べるつもりだった"),
        ],
    );
}

/// Bug: ``～しかない`` produced ``書きしかない`` (stem) instead of
/// ``書くしかない`` (dictionary form). ``しか～ない`` attaches to the noun form
/// of the verb, which for this construction is the plain dictionary form.
#[test]
fn shika_nai_uses_dictionary_form() {
    assert_rule_formats(
        "01KV2BV4G2TVKG953YH4390302",
        PartOfSpeech::Verb,
        &[("書く", "書くしかない"), ("食べる", "食べるしかない")],
    );
}

/// Bug: ``～たりしない`` produced ``飲んだたりしない`` for godan ``む`` verbs
/// because the ta-form was mis-conjugated. The godan ``む`` row maps to
/// ``んだ`` (not ``だた``) in ``TE_TA_MAPPING``, so ``飲む → 飲んだ → 飲んだり
/// → 飲んだりしない``.
#[test]
fn tari_shinai_correct_ta_form_for_godan_mu() {
    assert_rule_formats(
        "01KV2C261TX56MHY1TXVYM7CTB",
        PartOfSpeech::Verb,
        &[
            ("飲む", "飲んだりしない"), // CRITICAL: was 飲んだたりしない
            ("書く", "書いたりしない"),
            ("する", "したりしない"),
        ],
    );
}

// ===========================================================================
// 7 N5/N4 legacy fixes: dictionary form instead of stem
// ===========================================================================

/// Bug: ``～ことができます`` produced ``書きことができます`` (stem) instead of
/// ``書くことができます`` (dictionary form). ``ことができる`` attaches to the
/// dictionary form. The fix replaced ``VerbToMainView`` with a bare
/// ``AddPostfix``, matching the dictionary-form pattern.
#[test]
fn koto_ga_dekimasu_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000CM000000",
        PartOfSpeech::Verb,
        &[
            ("書く", "書くことができます"),
            ("食べる", "食べることができます"),
            ("する", "することができます"),
        ],
    );
}

/// Bug: ``～と思います`` produced ``書きと思います`` (stem). ``と`` quotative
/// particle attaches to the dictionary form.
#[test]
fn to_omoimasu_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000DW000000",
        PartOfSpeech::Verb,
        &[("書く", "書くと思います"), ("食べる", "食べると思います")],
    );
}

/// Bug: ``～と言います`` produced ``書きと言います`` (stem). Same dictionary-form
/// attachment as と思います.
#[test]
fn to_iimasu_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000E0000000",
        PartOfSpeech::Verb,
        &[("書く", "書くと言います"), ("食べる", "食べると言います")],
    );
}

/// Bug: ``～つもりです`` produced ``書きつもりです`` (stem). ``つもり``
/// attaches to the dictionary form (see also the N3 つもりはない fix above).
#[test]
fn tsumori_desu_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000MC000000",
        PartOfSpeech::Verb,
        &[("書く", "書くつもりです"), ("食べる", "食べるつもりです")],
    );
}

/// Bug: ``～ように`` produced ``書きように`` (stem). ``ように`` attaches to the
/// dictionary form for verbs.
#[test]
fn you_ni_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000QR000000",
        PartOfSpeech::Verb,
        &[("書く", "書くように"), ("食べる", "食べるように")],
    );
}

/// Bug: ``～ようになる`` produced ``書きようになる`` (stem). Same dictionary-form
/// attachment as ように.
#[test]
fn you_ni_naru_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000QW000000",
        PartOfSpeech::Verb,
        &[("書く", "書くようになる"), ("食べる", "食べるようになる")],
    );
}

/// Bug: ``～ようにする`` produced ``書きようにする`` (stem). Same dictionary-form
/// attachment as ように / ようになる.
#[test]
fn you_ni_suru_uses_dictionary_form() {
    assert_rule_formats(
        "01G000000000000000R0000000",
        PartOfSpeech::Verb,
        &[("書く", "書くようにする"), ("食べる", "食べるようにする")],
    );
}

// ===========================================================================
// Hearsay そうだ（伝聞）: plain form, NOT stem
// ===========================================================================

/// Bug: hearsay ``～そうだ（伝聞）`` produced ``降りそうだ`` for verbs (the
/// conjectural / 様態 form) instead of ``降るそうだ`` (plain form + そうだ).
///
/// The two そうだ are grammatically distinct:
///   - 様態そうだ (conjectural): attaches to verb stem → 降りそうだ ("looks
///     like it will rain")
///   - 伝聞そうだ (hearsay): attaches to plain form → 降るそうだ ("I heard
///     it will rain")
///
/// This rule (rule_id ``01G000000000000000W8000000``) is the HEARSAY
/// construction, so it must render the plain form before そうだ.
#[test]
fn hearsay_sou_da_uses_plain_form() {
    assert_rule_formats(
        "01G000000000000000W8000000",
        PartOfSpeech::Verb,
        &[
            ("降る", "降るそうだ"), // CRITICAL: was 降りそうだ
            ("書く", "書くそうだ"),
        ],
    );
    // Hearsay そうだ also attaches to い-adjectives in the plain form.
    assert_rule_formats(
        "01G000000000000000W8000000",
        PartOfSpeech::IAdjective,
        &[("美味しい", "美味しいそうだ")],
    );
}
