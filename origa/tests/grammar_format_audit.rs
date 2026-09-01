//! Grammar format_map audit harness.
//!
//! Run: `cargo test -p origa --test grammar_format_audit -- --nocapture --ignored`
//!
//! Loads cdn/grammar/grammar_v2.json, applies every format_map chain to 7 test
//! verbs spanning all verb groups, and dumps TSV results to stderr.

use std::path::PathBuf;

use origa::dictionary::grammar::{
    FormatAction, GrammarData, GrammarRule, init_grammar, is_grammar_loaded, iter_grammar_rules,
};

// godan: 書く(く), 話す(す), 待つ(つ), 飲む(む) | ichidan: 食べる | irregular: する, くる
const TEST_VERBS: &[&str] = &["書く", "食べる", "する", "くる", "話す", "待つ", "飲む"];

struct AuditRow {
    title: String,
    rule_id: String,
    pos: String,
    verb: String,
    chain: String,
    rendered: String,
    is_error: bool,
}

struct AuditData {
    rows: Vec<AuditRow>,
    double_te_regressions: Vec<String>,
    rules_with_format_map: usize,
}

fn cdn_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root parent of CARGO_MANIFEST_DIR")
        .join("cdn")
}

fn ensure_grammar_loaded() {
    if is_grammar_loaded() {
        return;
    }
    let grammar_path = cdn_dir().join("grammar").join("grammar_v2.json");
    let grammar_json =
        std::fs::read_to_string(&grammar_path).expect("grammar_v2.json must be readable");
    init_grammar(GrammarData { grammar_json }).expect("init_grammar must succeed");
}

fn rule_title(rule: &GrammarRule) -> String {
    let ru = rule
        .content(&origa::domain::NativeLanguage::Russian)
        .title();
    if !ru.trim().is_empty() {
        return ru.to_string();
    }
    rule.content(&origa::domain::NativeLanguage::English)
        .title()
        .to_string()
}

fn fmt_actions(actions: &[FormatAction]) -> String {
    actions
        .iter()
        .map(|a| match a {
            FormatAction::AddPostfix { postfix } => format!("AddPostfix({})", postfix),
            FormatAction::RemovePostfix { postfix } => format!("RemovePostfix({})", postfix),
            FormatAction::ReplacePostfix {
                old_postfix,
                new_postfix,
            } => format!("ReplacePostfix({} -> {})", old_postfix, new_postfix),
            other => format!("{:?}", other),
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

/// Structural double-て check: VerbToTeForm followed by AddPostfix whose
/// postfix starts with て/で. The te-form already ends in て/で, so such a
/// postfix yields a spurious double-て/で (書いててもらえませんか). Inspecting
/// the action chain is precise — a substring search on the rendered form would
/// false-positive on legitimate colloquial truncations like 書いてて.
fn has_double_te_regression(actions: &[FormatAction]) -> bool {
    for window in actions.windows(2) {
        if !matches!(window[0], FormatAction::VerbToTeForm {}) {
            continue;
        }
        if let FormatAction::AddPostfix { postfix } = &window[1] {
            if postfix.starts_with('て') || postfix.starts_with('で') {
                return true;
            }
        }
    }
    false
}

fn collect_evaluations() -> AuditData {
    ensure_grammar_loaded();

    let rules: Vec<&GrammarRule> = iter_grammar_rules().collect();
    let mut rows: Vec<AuditRow> = Vec::new();
    let mut double_te_regressions: Vec<String> = Vec::new();
    let mut rules_with_format_map = 0usize;

    for rule in rules {
        if !rule.has_format_map() {
            continue;
        }
        let apply_to = rule.apply_to();
        if apply_to.is_empty() {
            continue;
        }

        let title = rule_title(rule);
        let rule_id = rule.rule_id().to_string();
        rules_with_format_map += 1;

        for pos in &apply_to {
            let Some(actions) = rule.format_actions_for_pos(pos) else {
                continue;
            };
            if has_double_te_regression(actions) {
                double_te_regressions.push(format!(
                    "{} [{}]: {}",
                    title,
                    rule_id,
                    fmt_actions(actions)
                ));
            }
            let chain_str = fmt_actions(actions);
            for verb in TEST_VERBS {
                let (rendered, is_error) = match rule.format(verb, pos) {
                    Ok(s) => (s, false),
                    Err(e) => (format!("<ERR:{:?}>", e), true),
                };
                rows.push(AuditRow {
                    title: title.clone(),
                    rule_id: rule_id.clone(),
                    pos: format!("{:?}", pos),
                    verb: verb.to_string(),
                    chain: chain_str.clone(),
                    rendered,
                    is_error,
                });
            }
        }
    }

    AuditData {
        rows,
        double_te_regressions,
        rules_with_format_map,
    }
}

fn dump_evaluations(data: &AuditData) {
    eprintln!();
    eprintln!("title\trule_id\tpos\tverb\tchain\tresult");
    for row in &data.rows {
        eprintln!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.title, row.rule_id, row.pos, row.verb, row.chain, row.rendered
        );
    }
    let error_count = data.rows.iter().filter(|r| r.is_error).count();
    eprintln!();
    eprintln!(
        "AUDIT SUMMARY: {} rules with format_map × {} verbs = {} evaluations ({} errors)",
        data.rules_with_format_map,
        TEST_VERBS.len(),
        data.rows.len(),
        error_count
    );
}

#[test]
#[ignore = "audit harness: prints every format_map result, run explicitly"]
fn audit_all_format_map_chains() {
    let data = collect_evaluations();
    dump_evaluations(&data);

    let error_rows: Vec<&AuditRow> = data.rows.iter().filter(|r| r.is_error).collect();
    assert!(
        error_rows.is_empty(),
        "format_map produced errors for some verb/rule combos:\n{}",
        error_rows
            .iter()
            .map(|r| format!("{} [{}] {}: {}", r.title, r.rule_id, r.verb, r.rendered))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        data.double_te_regressions.is_empty(),
        "structural double-て/で regression (VerbToTeForm + AddPostfix starting with て/で):\n{}",
        data.double_te_regressions.join("\n")
    );
}
