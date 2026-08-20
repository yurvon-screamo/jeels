use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use std::fmt;
use std::str::FromStr;

use crate::loaders::precache_loader::PreCacheProgress;

const CARD_CACHE_STATE_KEY: &str = "__origa_card_cache_state__";

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CardCacheState {
    Idle,
    Running,
    Complete,
}

impl fmt::Display for CardCacheState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardCacheState::Idle => write!(f, "Idle"),
            CardCacheState::Running => write!(f, "Running"),
            CardCacheState::Complete => write!(f, "Complete"),
        }
    }
}

impl FromStr for CardCacheState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Idle" => Ok(CardCacheState::Idle),
            "Running" => Ok(CardCacheState::Running),
            "Complete" => Ok(CardCacheState::Complete),
            _ => Err(()),
        }
    }
}

/// Manages card cache state. Bundle download state is managed locally
/// in OfflineBundleCard since it's only needed in the profile page.
#[derive(Clone)]
pub struct OfflineBundleStore {
    pub card_cache_state: RwSignal<CardCacheState>,
    pub card_cache_progress: RwSignal<PreCacheProgress>,
}

impl OfflineBundleStore {
    fn load_state() -> CardCacheState {
        LocalStorage::get::<String>(CARD_CACHE_STATE_KEY)
            .ok()
            .and_then(|s| s.parse::<CardCacheState>().ok())
            .unwrap_or_else(|| {
                tracing::warn!("Failed to load card cache state from LocalStorage, starting fresh");
                CardCacheState::Idle
            })
    }

    fn save_state(state: CardCacheState) {
        if let Err(e) = LocalStorage::set(CARD_CACHE_STATE_KEY, state.to_string()) {
            tracing::warn!(error = %e, "Failed to save card cache state to LocalStorage");
        }
    }

    pub fn new() -> Self {
        let initial_state = Self::load_state();
        Self {
            card_cache_state: RwSignal::new(initial_state),
            card_cache_progress: RwSignal::new(PreCacheProgress::default()),
        }
    }

    pub fn set_card_cache_state(&self, state: CardCacheState) {
        self.card_cache_state.set(state);
        Self::save_state(state);
    }
}

impl Default for OfflineBundleStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_cache_state_display_and_fromstr() {
        assert_eq!(CardCacheState::Idle.to_string(), "Idle");
        assert_eq!(CardCacheState::Running.to_string(), "Running");
        assert_eq!(CardCacheState::Complete.to_string(), "Complete");

        assert_eq!("Idle".parse::<CardCacheState>(), Ok(CardCacheState::Idle));
        assert_eq!(
            "Running".parse::<CardCacheState>(),
            Ok(CardCacheState::Running)
        );
        assert_eq!(
            "Complete".parse::<CardCacheState>(),
            Ok(CardCacheState::Complete)
        );

        assert_eq!("invalid".parse::<CardCacheState>(), Err(()));
    }
}
