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

    pub fn is_complete(&self, threshold: f64) -> bool {
        self.percentage() >= threshold
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

    pub fn is_complete(&self, threshold: f64) -> bool {
        self.overall_percentage() >= threshold
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

impl JlptProgress {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        for level in JapaneseLevel::ALL {
            levels.insert(level, LevelProgressDetail::new());
        }
        Self { levels }
    }

    pub fn current_level(&self) -> JapaneseLevel {
        let all_empty = JapaneseLevel::ALL.iter().all(|&level| {
            self.levels
                .get(&level)
                .map(|d| d.overall_percentage() == 0.0)
                .unwrap_or(true)
        });

        if all_empty {
            return JapaneseLevel::N5;
        }

        let completed_levels: Vec<_> = JapaneseLevel::ALL
            .iter()
            .filter(|&&level| {
                self.levels
                    .get(&level)
                    .map(|d| d.is_complete(90.0))
                    .unwrap_or(false)
            })
            .collect();

        if completed_levels.is_empty() {
            return JapaneseLevel::N5;
        }

        let max_completed = completed_levels
            .into_iter()
            .min_by_key(|&&level| level.as_number())
            .unwrap();

        match max_completed {
            JapaneseLevel::N1 => JapaneseLevel::N1,
            JapaneseLevel::N2 => JapaneseLevel::N1,
            JapaneseLevel::N3 => JapaneseLevel::N2,
            JapaneseLevel::N4 => JapaneseLevel::N3,
            JapaneseLevel::N5 => JapaneseLevel::N4,
        }
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
    fn category_progress_is_complete_threshold() {
        let progress = CategoryProgress {
            learned: 90,
            projected: 0,
            total: 100,
        };
        assert!(progress.is_complete(90.0));
        assert!(!progress.is_complete(95.0));

        let progress_below = CategoryProgress {
            learned: 89,
            projected: 0,
            total: 100,
        };
        assert!(!progress_below.is_complete(90.0));
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

    #[test]
    fn jlpt_progress_current_level_n5_completed_returns_n4() {
        let mut progress = JlptProgress::new();
        let n5_complete = LevelProgressDetail {
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
                learned: 90,
                projected: 0,
                total: 100,
            },
        };
        progress.update_level(JapaneseLevel::N5, n5_complete);
        assert_eq!(progress.current_level(), JapaneseLevel::N4);
    }

    #[test]
    fn jlpt_progress_current_level_uses_learned_not_projected() {
        // Arrange — N5 has 0 learned but 100% projected. Projected must NOT
        // count toward `is_complete(90.0)` so the current level stays N5.
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
    fn jlpt_progress_current_level_n5_n4_completed_returns_n3() {
        let mut progress = JlptProgress::new();

        let complete = LevelProgressDetail {
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
                learned: 100,
                projected: 0,
                total: 100,
            },
        };

        progress.update_level(JapaneseLevel::N5, complete.clone());
        progress.update_level(JapaneseLevel::N4, complete);

        assert_eq!(progress.current_level(), JapaneseLevel::N3);
    }

    #[test]
    fn jlpt_progress_current_level_n1_completed_stays_n1() {
        let mut progress = JlptProgress::new();

        let complete = LevelProgressDetail {
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
                learned: 100,
                projected: 0,
                total: 100,
            },
        };

        for level in JapaneseLevel::ALL {
            progress.update_level(level, complete.clone());
        }

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
