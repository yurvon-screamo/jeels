use std::collections::{HashMap, HashSet};

use origa::domain::WellKnownSetMeta;
use origa::domain::{DailyLoad, JapaneseLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Intro,
    Load,
    Jlpt,
    Apps,
    Progress,
    Summary,
    Scoring,
}

impl OnboardingStep {
    pub fn as_usize(&self) -> usize {
        match self {
            OnboardingStep::Intro => 0,
            OnboardingStep::Load => 1,
            OnboardingStep::Jlpt => 2,
            OnboardingStep::Apps => 3,
            OnboardingStep::Progress => 4,
            OnboardingStep::Summary => 5,
            OnboardingStep::Scoring => 6,
        }
    }

    pub fn is_first(&self) -> bool {
        matches!(self, OnboardingStep::Intro)
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            OnboardingStep::Intro => Some(OnboardingStep::Load),
            OnboardingStep::Load => Some(OnboardingStep::Jlpt),
            OnboardingStep::Jlpt => Some(OnboardingStep::Apps),
            OnboardingStep::Apps => Some(OnboardingStep::Progress),
            OnboardingStep::Progress => Some(OnboardingStep::Summary),
            OnboardingStep::Summary => Some(OnboardingStep::Scoring),
            OnboardingStep::Scoring => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            OnboardingStep::Intro => None,
            OnboardingStep::Load => Some(OnboardingStep::Intro),
            OnboardingStep::Jlpt => Some(OnboardingStep::Load),
            OnboardingStep::Apps => Some(OnboardingStep::Jlpt),
            OnboardingStep::Progress => Some(OnboardingStep::Apps),
            OnboardingStep::Summary => Some(OnboardingStep::Progress),
            OnboardingStep::Scoring => Some(OnboardingStep::Summary),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
    pub current_step: OnboardingStep,
    pub selected_level: Option<JapaneseLevel>,
    pub selected_apps: HashSet<String>,
    pub apps_progress: HashMap<String, String>,
    pub sets_to_import: Vec<WellKnownSetMeta>,
    pub excluded_sets: HashSet<String>,
    pub available_sets: Vec<WellKnownSetMeta>,
    pub daily_load: DailyLoad,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            current_step: OnboardingStep::Intro,
            selected_level: None,
            selected_apps: HashSet::new(),
            apps_progress: HashMap::new(),
            sets_to_import: Vec::new(),
            excluded_sets: HashSet::new(),
            available_sets: Vec::new(),
            daily_load: DailyLoad::default(),
        }
    }
}

impl OnboardingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_app(&mut self, app_id: &str) {
        self.selected_apps.insert(app_id.to_string());
    }

    pub fn remove_app(&mut self, app_id: &str) {
        self.selected_apps.remove(app_id);
    }

    pub fn set_app_selection(&mut self, app_id: &str, set_id: &str) {
        self.apps_progress
            .insert(app_id.to_string(), set_id.to_string());
    }

    pub fn set_daily_load(&mut self, daily_load: DailyLoad) {
        self.daily_load = daily_load;
    }

    pub fn add_set_to_import(&mut self, set_meta: WellKnownSetMeta) {
        let set_id = set_meta.id.clone();
        if !self.excluded_sets.contains(&set_id)
            && !self.sets_to_import.iter().any(|s| s.id == set_id)
        {
            self.sets_to_import.push(set_meta);
        }
    }

    pub fn remove_set_from_import(&mut self, set_id: &str) {
        self.sets_to_import.retain(|s| s.id != set_id);
        self.excluded_sets.insert(set_id.to_string());
    }

    pub fn reset_exclusion(&mut self, set_id: &str) {
        self.excluded_sets.remove(set_id);
    }

    pub fn go_to_next_step(&mut self) -> bool {
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            true
        } else {
            false
        }
    }

    pub fn go_to_prev_step(&mut self) -> bool {
        if let Some(prev) = self.current_step.prev() {
            self.current_step = prev;
            true
        } else {
            false
        }
    }

    pub fn set_available_sets(&mut self, sets: Vec<WellKnownSetMeta>) {
        self.available_sets = sets;
    }

    pub fn get_final_sets(&self) -> Vec<String> {
        self.sets_to_import.iter().map(|s| s.id.clone()).collect()
    }

    pub fn is_first_step(&self) -> bool {
        self.current_step.is_first()
    }

    pub fn can_proceed(&self) -> bool {
        match self.current_step {
            OnboardingStep::Intro => true,
            OnboardingStep::Load => true,
            OnboardingStep::Jlpt => true,
            OnboardingStep::Apps => true,
            OnboardingStep::Progress => true,
            OnboardingStep::Summary => true,
            OnboardingStep::Scoring => true,
        }
    }

    fn is_jlpt_set_id(id: &str) -> bool {
        id.starts_with("jlpt_n")
    }

    pub fn clear_previous_jlpt_selections(&mut self) {
        self.sets_to_import.retain(|s| !Self::is_jlpt_set_id(&s.id));
    }

    pub fn set_jlpt_level(&mut self, level: Option<JapaneseLevel>) {
        self.selected_level = level;
        self.clear_previous_jlpt_selections();

        let levels_order = [
            JapaneseLevel::N5,
            JapaneseLevel::N4,
            JapaneseLevel::N3,
            JapaneseLevel::N2,
            JapaneseLevel::N1,
        ];

        if let Some(selected) = level {
            for &lvl in &levels_order {
                if lvl == selected {
                    break;
                }

                let set_id = format!("jlpt_{}", level_to_lowercase(lvl));
                if let Some(jlpt_set) = self.available_sets.iter().find(|s| s.id == set_id) {
                    self.sets_to_import.push(jlpt_set.clone());
                }
            }
        }
    }

    /// The JLPT level used to scope grammar and kanji imports. Prefers the
    /// level explicitly picked in `JlptStep`; otherwise falls back to the
    /// highest level among the selected sets so that, for example, choosing
    /// only Minna N4 still imports N4 grammar/kanji without a JLPT selection.
    ///
    /// NOTE: until `selected_level` is persisted into `user.jlpt_progress`
    /// (separate bug, out of scope), the fallback branch is always reached on
    /// a fresh app start. The branch is still exercised when the user has not
    /// picked a JLPT level in the current onboarding session.
    pub fn target_level(&self) -> JapaneseLevel {
        self.selected_level.unwrap_or_else(|| {
            self.sets_to_import
                .iter()
                .map(|s| s.level)
                .max()
                .unwrap_or(JapaneseLevel::N5)
        })
    }
}

fn level_to_lowercase(level: JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "n5",
        JapaneseLevel::N4 => "n4",
        JapaneseLevel::N3 => "n3",
        JapaneseLevel::N2 => "n2",
        JapaneseLevel::N1 => "n1",
    }
}
