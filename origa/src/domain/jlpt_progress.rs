use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::JapaneseLevel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryProgress {
    pub learned: usize,
    /// Cards that have been touched but not yet stabilized as learned —
    /// the sum of "in progress" and "hard" cards. Tracked separately from
    /// `learned` so the UI can render a "projected" progress layer on top
    /// of the learned layer. Old persisted data without this field defaults
    /// to `0`; it gets repopulated by `User::recalculate_jlpt_progress`.
    #[serde(default)]
    pub projected: usize,
    pub total: usize,
}

impl CategoryProgress {
    pub fn new() -> Self {
        Self {
            learned: 0,
            projected: 0,
            total: 0,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.learned as f64 / self.total as f64) * 100.0
    }

    /// Percentage counting learned + projected (everything except brand-new
    /// cards). Always `>= percentage()`. Used only for the visual "ghost"
    /// layer on the progress bar — does not affect level-up decisions.
    pub fn projected_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        ((self.learned + self.projected) as f64 / self.total as f64) * 100.0
    }
}

impl Default for CategoryProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelProgressDetail {
    pub kanji: CategoryProgress,
    pub words: CategoryProgress,
    pub grammar: CategoryProgress,
}

impl LevelProgressDetail {
    pub fn new() -> Self {
        Self {
            kanji: CategoryProgress::new(),
            words: CategoryProgress::new(),
            grammar: CategoryProgress::new(),
        }
    }

    pub fn overall_percentage(&self) -> f64 {
        (self.kanji.percentage() + self.words.percentage() + self.grammar.percentage()) / 3.0
    }

    /// Average of `projected_percentage()` across the three categories. Used
    /// only for the "ghost" visual layer. Always `>= overall_percentage()`.
    pub fn overall_projected_percentage(&self) -> f64 {
        (self.kanji.projected_percentage()
            + self.words.projected_percentage()
            + self.grammar.projected_percentage())
            / 3.0
    }
}

impl Default for LevelProgressDetail {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-category counts fed into [`JlptProgress::recalculate`]. Grouping the
/// 9 raw HashMaps into one struct keeps the recalculate signature small and
/// self-documenting.
#[derive(Debug, Clone, Default)]
pub struct CategoryCounts {
    pub kanji: HashMap<JapaneseLevel, usize>,
    pub words: HashMap<JapaneseLevel, usize>,
    pub grammar: HashMap<JapaneseLevel, usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ProgressUpdate {
    pub learned: CategoryCounts,
    pub projected: CategoryCounts,
    pub total: CategoryCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JlptProgress {
    levels: HashMap<JapaneseLevel, LevelProgressDetail>,
}

/// Minimum learned percentage of a level required before its knowledge may
/// overflow into the next one. Guards against inflation: 50% N3 + 50% N2
/// must not read as N2, so the base of each carried pair must be at least
/// this solid.
const ADVANCE_FLOOR_PERCENT: f64 = 70.0;

/// Combined learned percentage of a (level, next level) pair required to
/// count the level as passed. Textbook learners (Minna no Nihongo, Irodori)
/// rarely reach 90% inside a single official level because their vocabulary
/// only partially overlaps it, so completion is judged on the summed
/// knowledge of two adjacent levels instead.
const ADVANCE_SUM_PERCENT: f64 = 100.0;

/// The three progress categories tracked per JLPT level. Exists only to
/// select a category inside [`JlptProgress::carry_chain`] without generics:
/// a generic `impl Fn` selector would be monomorphized in dependent crates
/// and tips `origa_ui_bin` over its recursion limit (see ADR-027 §B3).
#[derive(Clone, Copy)]
enum Category {
    Kanji,
    Words,
    Grammar,
}

impl Category {
    fn percentage_of(self, detail: &LevelProgressDetail) -> f64 {
        match self {
            Category::Kanji => detail.kanji.percentage(),
            Category::Words => detail.words.percentage(),
            Category::Grammar => detail.grammar.percentage(),
        }
    }
}

impl JlptProgress {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        for level in JapaneseLevel::ALL {
            levels.insert(level, LevelProgressDetail::new());
        }
        Self { levels }
    }

    /// The user's active JLPT level, decided by knowledge overflow rather
    /// than per-level completion: each card category independently walks
    /// N5→N1 (see [`Self::carry_chain`]) and the reported level is the least
    /// advanced of the three chains, so one weak category cannot be
    /// compensated by another.
    pub fn current_level(&self) -> JapaneseLevel {
        self.carry_chain(Category::Kanji)
            .min(self.carry_chain(Category::Words))
            .min(self.carry_chain(Category::Grammar))
    }

    /// Walks `JapaneseLevel::ALL` in order and returns the level the category
    /// has overflowed into: advances from L to L+1 while `pct(L)` is at least
    /// [`ADVANCE_FLOOR_PERCENT`] and `pct(L) + pct(L+1)` covers
    /// [`ADVANCE_SUM_PERCENT`]. Levels are indexed by their position in
    /// `JapaneseLevel::ALL` — NOT by `as_number()`, which is inverted
    /// (N5 = 5 … N1 = 1).
    fn carry_chain(&self, category: Category) -> JapaneseLevel {
        let percentages: Vec<f64> = JapaneseLevel::ALL
            .iter()
            .map(|&level| {
                self.levels
                    .get(&level)
                    .map(|detail| category.percentage_of(detail))
                    .unwrap_or(0.0)
            })
            .collect();

        let last_index = percentages.len() - 1;
        let mut index = 0;
        while index < last_index {
            let current = percentages[index];
            let next = percentages[index + 1];
            if current >= ADVANCE_FLOOR_PERCENT && current + next >= ADVANCE_SUM_PERCENT {
                index += 1;
            } else {
                break;
            }
        }
        JapaneseLevel::ALL[index]
    }

    pub fn level_progress(&self, level: JapaneseLevel) -> Option<&LevelProgressDetail> {
        self.levels.get(&level)
    }

    pub fn update_level(&mut self, level: JapaneseLevel, detail: LevelProgressDetail) {
        self.levels.insert(level, detail);
    }

    pub fn recalculate(&mut self, update: ProgressUpdate) {
        for level in JapaneseLevel::ALL {
            let detail = LevelProgressDetail {
                kanji: CategoryProgress {
                    learned: *update.learned.kanji.get(&level).unwrap_or(&0),
                    projected: *update.projected.kanji.get(&level).unwrap_or(&0),
                    total: *update.total.kanji.get(&level).unwrap_or(&0),
                },
                words: CategoryProgress {
                    learned: *update.learned.words.get(&level).unwrap_or(&0),
                    projected: *update.projected.words.get(&level).unwrap_or(&0),
                    total: *update.total.words.get(&level).unwrap_or(&0),
                },
                grammar: CategoryProgress {
                    learned: *update.learned.grammar.get(&level).unwrap_or(&0),
                    projected: *update.projected.grammar.get(&level).unwrap_or(&0),
                    total: *update.total.grammar.get(&level).unwrap_or(&0),
                },
            };
            self.levels.insert(level, detail);
        }
    }
}

impl Default for JlptProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn category_progress_percentage_calculation() {
        let progress = CategoryProgress {
            learned: 50,
            projected: 0,
            total: 100,
        };
        assert!((progress.percentage() - 50.0).abs() < 0.001);

        let progress_zero_total = CategoryProgress {
            learned: 50,
            projected: 0,
            total: 0,
        };
        assert!((progress_zero_total.percentage() - 0.0).abs() < 0.001);

        let progress_full = CategoryProgress {
            learned: 100,
            projected: 0,
            total: 100,
        };
        assert!((progress_full.percentage() - 100.0).abs() < 0.001);
    }

    #[test]
    fn category_progress_projected_percentage_calculation() {
        // Arrange — learned=50, projected=30 (in-progress + hard), total=100
        let progress = CategoryProgress {
            learned: 50,
            projected: 30,
            total: 100,
        };

        // Assert — projected_percentage counts both layers
        assert!((progress.projected_percentage() - 80.0).abs() < 0.001);
        assert!(progress.projected_percentage() >= progress.percentage());
    }

    #[test]
    fn category_progress_projected_percentage_zero_total() {
        // Arrange
        let progress = CategoryProgress {
            learned: 50,
            projected: 30,
            total: 0,
        };

        // Assert
        assert!((progress.projected_percentage() - 0.0).abs() < 0.001);
    }

    #[test]
    fn level_progress_detail_overall_percentage() {
        let detail = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
            words: CategoryProgress {
                learned: 50,
                projected: 0,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 0,
                projected: 0,
                total: 100,
            },
        };
        assert!((detail.overall_percentage() - 50.0).abs() < 0.001);

        let detail_empty = LevelProgressDetail::new();
        assert!((detail_empty.overall_percentage() - 0.0).abs() < 0.001);
    }

    #[test]
    fn level_progress_detail_overall_projected_percentage() {
        // Arrange — three categories with both learned and projected counts
        let detail = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 40,
                projected: 10,
                total: 100,
            },
            words: CategoryProgress {
                learned: 60,
                projected: 20,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 50,
                projected: 0,
                total: 100,
            },
        };

        // Assert — projected layer adds (10+20+0)/3 = 10 on top of 50 learned average
        assert!((detail.overall_projected_percentage() - 60.0).abs() < 0.001);
        assert!(detail.overall_projected_percentage() >= detail.overall_percentage());
    }

    #[test]
    fn jlpt_progress_current_level_empty_returns_n5() {
        let progress = JlptProgress::new();
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    fn progress_with_categories(
        kanji: &[usize],
        words: &[usize],
        grammar: &[usize],
    ) -> JlptProgress {
        let category = |percentages: &[usize], index: usize| CategoryProgress {
            learned: percentages.get(index).copied().unwrap_or(0),
            projected: 0,
            total: 100,
        };
        let mut progress = JlptProgress::new();
        for (index, &level) in JapaneseLevel::ALL.iter().enumerate() {
            progress.update_level(
                level,
                LevelProgressDetail {
                    kanji: category(kanji, index),
                    words: category(words, index),
                    grammar: category(grammar, index),
                },
            );
        }
        progress
    }

    fn uniform_progress(percentages: &[usize]) -> JlptProgress {
        progress_with_categories(percentages, percentages, percentages)
    }

    #[rstest]
    #[case::textbook_spread(&[80, 40], JapaneseLevel::N4)]
    #[case::floor_boundary_inclusive(&[70, 30], JapaneseLevel::N4)]
    #[case::full_level_with_empty_next(&[100, 0], JapaneseLevel::N4)]
    #[case::multi_step(&[90, 80, 75], JapaneseLevel::N3)]
    fn jlpt_progress_current_level_carries_summed_knowledge_to_next_level(
        #[case] percentages: &[usize],
        #[case] expected: JapaneseLevel,
    ) {
        assert_eq!(
            uniform_progress(percentages).current_level(),
            expected,
            "percentages {percentages:?}"
        );
    }

    #[rstest]
    // Sum 125 ≥ 100, but the base never reaches the floor.
    #[case::weak_base(&[65, 60], JapaneseLevel::N5)]
    #[case::just_below_floor(&[69, 99], JapaneseLevel::N5)]
    // The textbook anti-case: 50% + 50% must not read as the next level.
    #[case::fifty_fifty(&[50, 50], JapaneseLevel::N5)]
    // Sequential chain, not greedy pair-scanning: N4 at 40% blocks N3 even
    // though 40 + 90 ≥ 100.
    #[case::mid_chain_gap(&[100, 40, 90], JapaneseLevel::N4)]
    #[case::inflated_upper_level(&[100, 100, 50, 50], JapaneseLevel::N3)]
    fn jlpt_progress_current_level_base_below_floor_blocks_advance(
        #[case] percentages: &[usize],
        #[case] expected: JapaneseLevel,
    ) {
        assert_eq!(
            uniform_progress(percentages).current_level(),
            expected,
            "percentages {percentages:?}"
        );
    }

    #[rstest]
    #[case::partial_next(&[80, 15], JapaneseLevel::N5)]
    #[case::almost_full_base(&[95, 4], JapaneseLevel::N5)]
    fn jlpt_progress_current_level_sum_below_threshold_keeps_level(
        #[case] percentages: &[usize],
        #[case] expected: JapaneseLevel,
    ) {
        assert_eq!(
            uniform_progress(percentages).current_level(),
            expected,
            "percentages {percentages:?}"
        );
    }

    #[test]
    fn jlpt_progress_current_level_weak_category_cannot_be_compensated() {
        // Arrange — kanji and words at 100% N5, grammar at 40%: the old
        // average (80%) and the min-of-chains both keep N5, but only the
        // chain semantics guarantees grammar alone is the blocker.
        let progress = progress_with_categories(&[100], &[100], &[40]);

        // Act & Assert
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    #[test]
    fn jlpt_progress_current_level_reports_least_advanced_category_chain() {
        // Arrange — kanji carries to N3, words to N4, grammar stays at N5.
        let progress = progress_with_categories(&[100, 100, 80], &[100, 50], &[30]);

        // Act & Assert
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    #[test]
    fn jlpt_progress_current_level_one_category_short_of_sum_keeps_level() {
        // Arrange — grammar at 90% alone: above the floor, but 90 + 0 < 100.
        // Under the retired average-threshold rule this user was promoted
        // (avg 96.7%); the carry rule requires summed knowledge instead.
        let progress = progress_with_categories(&[100], &[100], &[90]);

        // Act & Assert
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    #[test]
    fn jlpt_progress_current_level_zero_total_level_caps_category_chain() {
        // Arrange — grammar content exists only for N5; N4+ totals are empty
        // (percentage reads as 0), while kanji and words are fully learned.
        let mut progress = JlptProgress::new();
        for (index, &level) in JapaneseLevel::ALL.iter().enumerate() {
            let grammar_learned = if index == 0 { 100 } else { 0 };
            let grammar_total = if index == 0 { 100 } else { 0 };
            progress.update_level(
                level,
                LevelProgressDetail {
                    kanji: CategoryProgress {
                        learned: 100,
                        projected: 0,
                        total: 100,
                    },
                    words: CategoryProgress {
                        learned: 100,
                        projected: 0,
                        total: 100,
                    },
                    grammar: CategoryProgress {
                        learned: grammar_learned,
                        projected: 0,
                        total: grammar_total,
                    },
                },
            );
        }

        // Act & Assert — kanji/words chains reach N1; the grammar chain may
        // step into the empty N4 (100 + 0 ≥ 100) but never past it (0 is
        // below the floor), so the reported level is N4.
        assert_eq!(progress.current_level(), JapaneseLevel::N4);
    }

    #[test]
    fn jlpt_progress_current_level_uses_learned_not_projected() {
        // Arrange — N5 has 0 learned but 100% projected. Projected must NOT
        // count toward the carry advance, so the current level stays N5.
        let mut progress = JlptProgress::new();
        let n5_only_projected = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 0,
                projected: 100,
                total: 100,
            },
            words: CategoryProgress {
                learned: 0,
                projected: 100,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 0,
                projected: 100,
                total: 100,
            },
        };
        progress.update_level(JapaneseLevel::N5, n5_only_projected);

        // Assert — current_level only considers learned, so we are still at N5
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    #[test]
    fn jlpt_progress_current_level_two_full_levels_carries_into_third() {
        let progress = uniform_progress(&[100, 100]);

        assert_eq!(progress.current_level(), JapaneseLevel::N3);
    }

    #[test]
    fn jlpt_progress_current_level_full_knowledge_stays_at_n1() {
        let progress = uniform_progress(&[100, 100, 100, 100, 100]);

        assert_eq!(progress.current_level(), JapaneseLevel::N1);
    }

    #[test]
    fn jlpt_progress_recalculate_updates_all_levels() {
        // Arrange
        let mut progress = JlptProgress::new();

        let mut learned = CategoryCounts::default();
        learned.kanji.insert(JapaneseLevel::N5, 50);
        learned.kanji.insert(JapaneseLevel::N4, 30);
        learned.words.insert(JapaneseLevel::N5, 100);
        learned.grammar.insert(JapaneseLevel::N5, 25);

        let mut projected = CategoryCounts::default();
        projected.kanji.insert(JapaneseLevel::N5, 10);
        projected.words.insert(JapaneseLevel::N5, 20);

        let mut total = CategoryCounts::default();
        total.kanji.insert(JapaneseLevel::N5, 100);
        total.kanji.insert(JapaneseLevel::N4, 150);
        total.words.insert(JapaneseLevel::N5, 200);
        total.grammar.insert(JapaneseLevel::N5, 50);

        let update = ProgressUpdate {
            learned,
            projected,
            total,
        };

        // Act
        progress.recalculate(update);

        // Assert — N5 fully populated
        let n5_progress = progress.level_progress(JapaneseLevel::N5).unwrap();
        assert_eq!(n5_progress.kanji.learned, 50);
        assert_eq!(n5_progress.kanji.projected, 10);
        assert_eq!(n5_progress.kanji.total, 100);
        assert_eq!(n5_progress.words.learned, 100);
        assert_eq!(n5_progress.words.projected, 20);
        assert_eq!(n5_progress.words.total, 200);
        assert_eq!(n5_progress.grammar.learned, 25);
        assert_eq!(n5_progress.grammar.projected, 0);
        assert_eq!(n5_progress.grammar.total, 50);

        // Assert — N4 only kanji populated
        let n4_progress = progress.level_progress(JapaneseLevel::N4).unwrap();
        assert_eq!(n4_progress.kanji.learned, 30);
        assert_eq!(n4_progress.kanji.total, 150);
        assert_eq!(n4_progress.words.learned, 0);
        assert_eq!(n4_progress.grammar.learned, 0);
    }
}
