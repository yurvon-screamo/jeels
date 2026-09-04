use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use tracing::info;
use ulid::Ulid;

use crate::dictionary::grammar::{get_all_rule_ids, is_grammar_loaded};
use crate::dictionary::phrase::{
    get_all_index_ids, get_index_entry, get_phrases_by_token, is_phrases_loaded,
};
use crate::domain::{Card, OrigaError, PhraseCard, StudyCard};
use crate::traits::UserRepository;

#[derive(Clone)]
pub struct SeedReadyPhrasesUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SeedReadyPhrasesUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<usize, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let grammar_orphaned_removed = remove_orphaned_grammar_cards(&mut user);

        if !is_phrases_loaded() {
            if grammar_orphaned_removed > 0 {
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let orphaned_removed = remove_orphaned_phrase_cards(&mut user);

        let (known_words, known_grammar) = {
            let study_cards = user.knowledge_set().study_cards();
            (
                crate::domain::collect_known_vocabulary_words(study_cards.values(), true),
                collect_known_grammar_rules(study_cards),
            )
        };

        if known_words.is_empty() {
            if orphaned_removed > 0 || grammar_orphaned_removed > 0 {
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let current_hash = compute_combined_hash(&known_words, &known_grammar);
        if current_hash == user.known_vocab_hash() {
            if orphaned_removed > 0 || grammar_orphaned_removed > 0 {
                user.set_known_vocab_hash(current_hash);
                self.repository.save(&user).await?;
            }
            return Ok(0);
        }

        let study_cards = user.knowledge_set().study_cards();
        let existing_phrase_ids = collect_existing_phrase_ids(study_cards);
        let ready_phrase_ids =
            find_ready_phrases(&known_words, &known_grammar, &existing_phrase_ids);

        let mut created_count = 0;
        for phrase_id in &ready_phrase_ids {
            let phrase_card = PhraseCard::new(*phrase_id);
            if user.create_card(Card::Phrase(phrase_card)).is_ok() {
                created_count += 1;
            }
        }

        user.set_known_vocab_hash(current_hash);
        self.repository.save(&user).await?;

        info!(
            created = created_count,
            orphaned_removed = orphaned_removed,
            grammar_orphaned_removed = grammar_orphaned_removed,
            known_vocab_count = known_words.len(),
            "Seeded ready phrase cards"
        );

        Ok(created_count)
    }
}

pub fn collect_known_grammar_rules(study_cards: &HashMap<Ulid, StudyCard>) -> HashSet<Ulid> {
    study_cards
        .values()
        .filter_map(|sc| {
            if let Card::Grammar(grammar_card) = sc.card() {
                if sc.memory().is_in_progress() || sc.memory().is_known_card() {
                    return Some(*grammar_card.rule_id());
                }
            }
            None
        })
        .collect()
}

fn compute_combined_hash(known_words: &HashSet<String>, known_grammar: &HashSet<Ulid>) -> u32 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();

    let mut words: Vec<&String> = known_words.iter().collect();
    words.sort();
    for word in &words {
        word.hash(&mut hasher);
    }

    let mut rules: Vec<&Ulid> = known_grammar.iter().collect();
    rules.sort();
    for rule in &rules {
        rule.hash(&mut hasher);
    }

    hasher.finish() as u32
}

fn collect_existing_phrase_ids(study_cards: &HashMap<Ulid, StudyCard>) -> HashSet<Ulid> {
    study_cards
        .values()
        .filter_map(|sc| {
            if let Card::Phrase(phrase_card) = sc.card() {
                Some(*phrase_card.phrase_id())
            } else {
                None
            }
        })
        .collect()
}

fn remove_orphaned_phrase_cards(user: &mut crate::domain::User) -> usize {
    if !is_phrases_loaded() {
        return 0;
    }

    let valid_ids = get_all_index_ids();

    // Collect known grammar rules BEFORE borrowing `user` for the iteration below.
    // The helper returns an owned `HashSet`, so the immutable borrow of `user`
    // ends here and the loop can subsequently borrow it again (NLL).
    let known_grammar = collect_known_grammar_rules(user.knowledge_set().study_cards());

    let orphaned_card_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, sc)| {
            if let Card::Phrase(phrase_card) = sc.card() {
                let phrase_id = phrase_card.phrase_id();

                // Check 1: stale — phrase_id is no longer present in the index.
                if !valid_ids.contains(phrase_id) {
                    return Some(*card_id);
                }

                // Check 2: grammar-ineligible — the phrase references grammar
                // rules the user has not learned yet. Such cards may have been
                // created under an older phrase_index where N3/N2 grammar was
                // not detected (g: []); once the index is re-enriched they must
                // be cleaned up.
                if let Some(entry) = get_index_entry(phrase_id) {
                    let grammar_eligible = entry
                        .grammar_rules()
                        .iter()
                        .all(|rule_id| known_grammar.contains(rule_id));
                    if !grammar_eligible {
                        return Some(*card_id);
                    }
                }
                // If the entry is somehow not loadable (defensive — should not
                // happen when `valid_ids` contains the id), keep the card.
            }
            None
        })
        .collect();

    let count = orphaned_card_ids.len();
    if count > 0 {
        info!(count, "Removing orphaned/grammar-ineligible phrase cards");
        for card_id in &orphaned_card_ids {
            if user.delete_card(*card_id).is_err() {
                tracing::warn!(%card_id, "Failed to delete orphaned phrase card");
            }
        }
    }
    count
}

fn remove_orphaned_grammar_cards(user: &mut crate::domain::User) -> usize {
    if !is_grammar_loaded() {
        return 0;
    }

    let valid_ids = get_all_rule_ids();

    let orphaned_card_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, sc)| {
            if let Card::Grammar(grammar_card) = sc.card() {
                if !valid_ids.contains(grammar_card.rule_id()) {
                    return Some(*card_id);
                }
            }
            None
        })
        .collect();

    let count = orphaned_card_ids.len();
    if count > 0 {
        info!(count, "Removing orphaned grammar cards");
        for card_id in &orphaned_card_ids {
            if user.delete_card(*card_id).is_err() {
                tracing::warn!(%card_id, "Failed to delete orphaned grammar card");
            }
        }
    }
    count
}

fn find_ready_phrases(
    known_words: &HashSet<String>,
    known_grammar: &HashSet<Ulid>,
    existing_phrase_ids: &HashSet<Ulid>,
) -> Vec<Ulid> {
    let mut seen_phrase_ids: HashSet<Ulid> = HashSet::new();
    let mut result: Vec<Ulid> = Vec::new();

    for word in known_words {
        for entry in get_phrases_by_token(word) {
            let phrase_id = entry.id();
            if seen_phrase_ids.contains(phrase_id) || existing_phrase_ids.contains(phrase_id) {
                continue;
            }
            seen_phrase_ids.insert(*phrase_id);

            let all_tokens_known = entry
                .tokens()
                .iter()
                .all(|token| known_words.contains(token));

            if !all_tokens_known {
                continue;
            }

            let grammar_ready = entry
                .grammar_rules()
                .iter()
                .all(|rule_id| known_grammar.contains(rule_id));

            if !grammar_ready {
                continue;
            }

            result.push(*phrase_id);
        }
    }

    result
}

pub fn classify_orphaned_phrases(failed_phrase_ids: &[Ulid]) -> (HashSet<Ulid>, HashSet<Ulid>) {
    use crate::dictionary::phrase::{get_chunk_id, is_chunk_loaded};

    let permanent: HashSet<Ulid> = failed_phrase_ids
        .iter()
        .filter(|id| match get_chunk_id(id) {
            None => true,
            Some(chunk_id) => is_chunk_loaded(chunk_id),
        })
        .copied()
        .collect();

    let transient: HashSet<Ulid> = failed_phrase_ids
        .iter()
        .copied()
        .filter(|id| !permanent.contains(id))
        .collect();

    (permanent, transient)
}

pub fn delete_phrase_cards_by_phrase_ids(
    user: &mut crate::domain::User,
    phrase_ids: &HashSet<Ulid>,
) -> usize {
    let card_ids_to_delete: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .iter()
        .filter_map(|(card_id, sc)| {
            if let Card::Phrase(phrase_card) = sc.card() {
                if phrase_ids.contains(phrase_card.phrase_id()) {
                    return Some(*card_id);
                }
            }
            None
        })
        .collect();

    let count = card_ids_to_delete.len();
    if count > 0 {
        for card_id in &card_ids_to_delete {
            if user.delete_card(*card_id).is_err() {
                tracing::warn!(%card_id, "Failed to delete orphaned phrase card");
            }
        }
        info!(count, "Deleted orphaned phrase cards by phrase ID");
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{GrammarData, init_grammar};
    use crate::dictionary::phrase::init_phrase_index;
    use crate::domain::{GrammarRuleCard, NativeLanguage, PhraseCard, User, VocabularyCard};
    use std::sync::{Once, OnceLock};

    static INIT: Once = Once::new();

    fn ensure_grammar_loaded() {
        INIT.call_once(|| {
            if is_grammar_loaded() {
                return;
            }

            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

            let grammar_path = std::path::PathBuf::from(manifest_dir)
                .parent()
                .expect("Failed to get parent directory")
                .join("cdn")
                .join("grammar")
                .join("grammar_v2.json");

            let grammar_json =
                std::fs::read_to_string(&grammar_path).expect("Failed to read grammar_v2.json");

            let grammar_data = GrammarData { grammar_json };
            init_grammar(grammar_data).expect("Failed to init grammar rules");
        });
    }

    // Phrase index is a process-wide `OnceLock`; it can only be initialized once.
    // To remain compatible with `journeys/phrase.rs` (which may have already
    // initialized it under the same test binary), we reuse the exact same test
    // fixture (same ULIDs) so the assertions hold regardless of who wins the
    // initialization race.
    static PHRASE_INIT: OnceLock<()> = OnceLock::new();

    // Phrase without grammar rules — `g` field absent.
    fn phrase_id_without_grammar() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
    }

    // Phrase referencing a single grammar rule. Eligibility depends on the
    // user's known set — same fixture is reused for "removed when unknown"
    // and "kept when known" tests, so the name describes the data shape only.
    fn phrase_id_with_single_grammar_rule() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").expect("valid ULID")
    }

    fn grammar_rule_id_for_bye() -> Ulid {
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("valid ULID")
    }

    fn ensure_phrases_loaded() {
        PHRASE_INIT.get_or_init(|| {
            if is_phrases_loaded() {
                return;
            }
            let index_json = r#"{"v":1,"h":"test","total":8,"phrases":[
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB","01G00000000000000024000000"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HP","t":["test","は"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HQ","t":["hello","extra1"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HR","t":["hello","extra2"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HS","t":["alpha","beta"],"c":0}
            ]}"#;
            init_phrase_index(index_json).expect("Failed to init phrase index");
        });
    }

    fn create_test_user() -> User {
        User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        )
    }

    fn get_first_rule_id() -> Ulid {
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    #[test]
    fn remove_orphaned_grammar_cards_returns_zero_when_no_orphaned_cards() {
        ensure_grammar_loaded();

        let mut user = create_test_user();
        let valid_rule_id = get_first_rule_id();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            valid_rule_id,
        )))
        .unwrap();

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 0);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);
    }

    #[test]
    fn remove_orphaned_grammar_cards_removes_cards_with_nonexistent_rule_ids() {
        ensure_grammar_loaded();

        let mut user = create_test_user();

        let valid_rule_id = get_first_rule_id();
        let orphan_rule_id = Ulid::new();

        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            valid_rule_id,
        )))
        .unwrap();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();

        assert_eq!(user.knowledge_set().study_cards().len(), 2);

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 1);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);

        let remaining_card = user.knowledge_set().study_cards().values().next().unwrap();
        if let Card::Grammar(grammar_card) = remaining_card.card() {
            assert_eq!(*grammar_card.rule_id(), valid_rule_id);
        } else {
            panic!("Expected grammar card");
        }
    }

    #[test]
    fn remove_orphaned_grammar_cards_returns_zero_when_grammar_not_loaded() {
        let mut user = create_test_user();
        let orphan_rule_id = Ulid::new();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();

        // This test may run before or after grammar is loaded.
        // If grammar is already loaded, the card would be orphaned.
        // The key behavior being tested: function does not panic.
        let removed = remove_orphaned_grammar_cards(&mut user);

        if !is_grammar_loaded() {
            assert_eq!(removed, 0);
            assert_eq!(user.knowledge_set().study_cards().len(), 1);
        }
        // If grammar IS loaded (from another test), the card would be orphaned.
        // We don't assert in that case since it's environment-dependent.
    }

    #[test]
    fn remove_orphaned_grammar_cards_ignores_non_grammar_cards() {
        ensure_grammar_loaded();

        let mut user = create_test_user();
        let orphan_rule_id = Ulid::new();

        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            orphan_rule_id,
        )))
        .unwrap();
        user.create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .unwrap();

        let removed = remove_orphaned_grammar_cards(&mut user);

        assert_eq!(removed, 1);

        let remaining = user.knowledge_set().study_cards();
        assert_eq!(remaining.len(), 1);
        assert!(
            remaining
                .values()
                .any(|sc| matches!(sc.card(), Card::Phrase(_)))
        );
    }

    fn add_known_grammar_card(user: &mut User, rule_id: Ulid) {
        let study_card = user
            .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
            .expect("Failed to create grammar card");
        user.knowledge_set_mut()
            .mark_card_as_known(*study_card.card_id())
            .expect("Failed to mark grammar card as known");
    }

    fn study_cards_contains_phrase(user: &User, phrase_id: Ulid) -> bool {
        user.knowledge_set()
            .study_cards()
            .values()
            .any(|sc| matches!(sc.card(), Card::Phrase(p) if *p.phrase_id() == phrase_id))
    }

    #[test]
    fn remove_orphaned_phrase_cards_removes_grammar_ineligible() {
        ensure_phrases_loaded();

        let mut user = create_test_user();
        // User has NO grammar cards — the rule referenced by this phrase is unknown.
        user.create_card(Card::Phrase(PhraseCard::new(
            phrase_id_with_single_grammar_rule(),
        )))
        .unwrap();

        let removed = remove_orphaned_phrase_cards(&mut user);

        assert_eq!(
            removed, 1,
            "Phrase card with unknown grammar rule must be removed"
        );
        assert!(
            !study_cards_contains_phrase(&user, phrase_id_with_single_grammar_rule()),
            "Grammar-ineligible phrase card must no longer be present"
        );
    }

    #[test]
    fn remove_orphaned_phrase_cards_keeps_grammar_eligible() {
        ensure_phrases_loaded();

        let mut user = create_test_user();
        // User knows the grammar rule referenced by the phrase.
        add_known_grammar_card(&mut user, grammar_rule_id_for_bye());
        user.create_card(Card::Phrase(PhraseCard::new(
            phrase_id_with_single_grammar_rule(),
        )))
        .unwrap();

        let removed = remove_orphaned_phrase_cards(&mut user);

        assert_eq!(removed, 0, "Grammar-eligible phrase card must be kept");
        assert!(
            study_cards_contains_phrase(&user, phrase_id_with_single_grammar_rule()),
            "Phrase card with all-known grammar must remain"
        );
    }

    #[test]
    fn remove_orphaned_phrase_cards_keeps_phrases_without_grammar_rules() {
        ensure_phrases_loaded();

        let mut user = create_test_user();
        // Phrase has `g: []` (absent) — vacuously grammar-eligible.
        user.create_card(Card::Phrase(PhraseCard::new(phrase_id_without_grammar())))
            .unwrap();

        let removed = remove_orphaned_phrase_cards(&mut user);

        assert_eq!(removed, 0, "Phrase with no grammar rules must be kept");
        assert!(
            study_cards_contains_phrase(&user, phrase_id_without_grammar()),
            "Phrase card without grammar rules must remain"
        );
    }

    #[test]
    fn remove_orphaned_phrase_cards_removes_stale_phrase_id() {
        ensure_phrases_loaded();

        let mut user = create_test_user();
        let stale_phrase_id = Ulid::new();
        user.create_card(Card::Phrase(PhraseCard::new(stale_phrase_id)))
            .unwrap();

        let removed = remove_orphaned_phrase_cards(&mut user);

        assert_eq!(
            removed, 1,
            "Phrase card with unknown phrase_id must be removed"
        );
        assert!(
            !study_cards_contains_phrase(&user, stale_phrase_id),
            "Stale phrase card must no longer be present"
        );
    }

    #[test]
    fn remove_orphaned_phrase_cards_returns_zero_when_phrases_not_loaded() {
        // The phrase index is a process-wide `OnceLock`; other tests in the
        // same binary (e.g. `journeys::phrase`) may initialize it concurrently.
        // The only behavior we can deterministically verify without serializing
        // is that the function does not panic. State-dependent assertions only
        // apply when the index is still unloaded AFTER the call — mirroring
        // the contract of `remove_orphaned_grammar_cards_returns_zero_when_grammar_not_loaded`.
        let mut user = create_test_user();
        user.create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .unwrap();

        let removed = remove_orphaned_phrase_cards(&mut user);

        if !is_phrases_loaded() {
            assert_eq!(removed, 0);
            assert_eq!(user.knowledge_set().study_cards().len(), 1);
        }
    }

    #[test]
    fn collect_known_grammar_rules_returns_empty_for_no_grammar_cards() {
        let mut user = create_test_user();
        user.create_card(Card::Vocabulary(crate::domain::VocabularyCard::new(
            crate::domain::value_objects::Question::new("test".to_string()).unwrap(),
        )))
        .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert!(result.is_empty());
    }

    #[test]
    fn collect_known_grammar_rules_returns_ids_for_known_grammar_cards() {
        let mut user = create_test_user();
        let rule_id = Ulid::new();
        let study_card = user
            .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
            .unwrap();
        user.knowledge_set_mut()
            .mark_card_as_known(*study_card.card_id())
            .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&rule_id));
    }

    #[test]
    fn collect_known_grammar_rules_excludes_unknown_grammar_cards() {
        let mut user = create_test_user();
        let rule_id = Ulid::new();
        user.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
            .unwrap();

        let study_cards = user.knowledge_set().study_cards();
        let result = collect_known_grammar_rules(study_cards);

        assert!(
            result.is_empty(),
            "New (not-yet-known) grammar cards should be excluded"
        );
    }

    #[test]
    fn compute_combined_hash_differs_when_grammar_changes() {
        let words: HashSet<String> = ["hello".to_string(), "test".to_string()]
            .into_iter()
            .collect();

        let grammar_empty: HashSet<Ulid> = HashSet::new();
        let grammar_with_rule: HashSet<Ulid> = [Ulid::new()].into_iter().collect();

        let hash_empty = compute_combined_hash(&words, &grammar_empty);
        let hash_with_rule = compute_combined_hash(&words, &grammar_with_rule);

        assert_ne!(
            hash_empty, hash_with_rule,
            "Hash should change when grammar rules are added"
        );
    }

    #[test]
    fn compute_combined_hash_same_for_identical_inputs() {
        let words: HashSet<String> = ["hello".to_string()].into_iter().collect();
        let rules: HashSet<Ulid> = [Ulid::from_string("01KPG000000000000000000001").unwrap()]
            .into_iter()
            .collect();

        let hash1 = compute_combined_hash(&words, &rules);
        let hash2 = compute_combined_hash(&words, &rules);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn compute_combined_hash_differs_from_words_only_hash() {
        let words: HashSet<String> = ["hello".to_string()].into_iter().collect();
        let rules: HashSet<Ulid> = [Ulid::new()].into_iter().collect();

        let hash_words_only = compute_combined_hash(&words, &HashSet::new());
        let hash_with_grammar = compute_combined_hash(&words, &rules);

        assert_ne!(
            hash_words_only, hash_with_grammar,
            "Combined hash should differ from words-only hash"
        );
    }

    #[test]
    fn classify_orphaned_phrases_not_in_index_is_permanent() {
        let unknown_id = Ulid::new();
        let (permanent, transient) = classify_orphaned_phrases(&[unknown_id]);
        assert!(permanent.contains(&unknown_id));
        assert!(transient.is_empty());
    }

    #[test]
    fn classify_orphaned_phrases_empty_input() {
        let (permanent, transient) = classify_orphaned_phrases(&[]);
        assert!(permanent.is_empty());
        assert!(transient.is_empty());
    }

    #[test]
    fn delete_phrase_cards_by_phrase_ids_removes_matching() {
        let mut user = create_test_user();
        let phrase_id_1 = Ulid::new();
        let phrase_id_2 = Ulid::new();

        user.create_card(Card::Phrase(PhraseCard::new(phrase_id_1)))
            .unwrap();
        user.create_card(Card::Phrase(PhraseCard::new(phrase_id_2)))
            .unwrap();
        user.create_card(Card::Vocabulary(VocabularyCard::new(
            crate::domain::value_objects::Question::new("test".to_string()).unwrap(),
        )))
        .unwrap();

        let to_delete: HashSet<Ulid> = [phrase_id_1].into_iter().collect();
        let deleted = delete_phrase_cards_by_phrase_ids(&mut user, &to_delete);

        assert_eq!(deleted, 1);
        assert_eq!(user.knowledge_set().study_cards().len(), 2);
        assert!(
            user.knowledge_set().study_cards().values().any(|sc| {
                matches!(sc.card(), Card::Phrase(pc) if *pc.phrase_id() == phrase_id_2)
            })
        );
    }

    #[test]
    fn delete_phrase_cards_by_phrase_ids_no_match() {
        let mut user = create_test_user();
        let phrase_id = Ulid::new();
        user.create_card(Card::Phrase(PhraseCard::new(phrase_id)))
            .unwrap();

        let to_delete: HashSet<Ulid> = [Ulid::new()].into_iter().collect();
        let deleted = delete_phrase_cards_by_phrase_ids(&mut user, &to_delete);

        assert_eq!(deleted, 0);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);
    }

    #[test]
    fn delete_phrase_cards_by_phrase_ids_ignores_non_phrase_cards() {
        let mut user = create_test_user();
        user.create_card(Card::Vocabulary(VocabularyCard::new(
            crate::domain::value_objects::Question::new("test".to_string()).unwrap(),
        )))
        .unwrap();

        let to_delete: HashSet<Ulid> = [Ulid::new()].into_iter().collect();
        let deleted = delete_phrase_cards_by_phrase_ids(&mut user, &to_delete);

        assert_eq!(deleted, 0);
        assert_eq!(user.knowledge_set().study_cards().len(), 1);
    }
}
