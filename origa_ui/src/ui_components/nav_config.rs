#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NavRoute {
    Home,
    Words,
    Grammar,
    Kanji,
    Phrases,
    Profile,
}

impl NavRoute {
    pub const fn href(self) -> &'static str {
        match self {
            Self::Home => "/home",
            Self::Words => "/words",
            Self::Grammar => "/grammar",
            Self::Kanji => "/kanji",
            Self::Phrases => "/phrases",
            Self::Profile => "/profile",
        }
    }

    pub const fn icon(self) -> icondata::Icon {
        match self {
            Self::Home => icondata::LuHouse,
            Self::Words => icondata::LuLanguages,
            Self::Grammar => icondata::LuPencilLine,
            Self::Kanji => icondata::LuBookOpen,
            Self::Phrases => icondata::LuMessageSquare,
            Self::Profile => icondata::LuUser,
        }
    }

    pub const fn use_logo(self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn label(self, i18n: &leptos_i18n::I18nContext<crate::i18n::Locale>) -> String {
        let keys = i18n.get_keys();
        match self {
            Self::Home => keys.home().home_tab().inner().to_string(),
            Self::Words => keys.home().words().inner().to_string(),
            Self::Grammar => keys.home().grammar().inner().to_string(),
            Self::Kanji => keys.home().kanji().inner().to_string(),
            Self::Phrases => keys.home().phrases().inner().to_string(),
            Self::Profile => keys.home().profile().inner().to_string(),
        }
    }

    pub fn is_active(self, path: &str) -> bool {
        match self {
            Self::Home => path.starts_with("/home") || path == "/" || path.is_empty(),
            Self::Words => path.starts_with("/words") || path.starts_with("/sets"),
            Self::Grammar => path.starts_with("/grammar"),
            Self::Kanji => path.starts_with("/kanji"),
            Self::Phrases => path.starts_with("/phrases"),
            Self::Profile => path.starts_with("/profile"),
        }
    }

    pub const fn all() -> &'static [NavRoute; 6] {
        &[
            Self::Home,
            Self::Words,
            Self::Grammar,
            Self::Kanji,
            Self::Phrases,
            Self::Profile,
        ]
    }

    pub const fn sidebar_routes() -> &'static [NavRoute; 5] {
        &[
            Self::Home,
            Self::Words,
            Self::Grammar,
            Self::Kanji,
            Self::Phrases,
        ]
    }

    pub const fn test_id_suffix(self) -> &'static str {
        match self {
            Self::Home => "tab-home",
            Self::Words => "tab-words",
            Self::Grammar => "tab-grammar",
            Self::Kanji => "tab-kanji",
            Self::Phrases => "tab-phrases",
            Self::Profile => "tab-profile",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn href_maps_every_route() {
        assert_eq!(NavRoute::Home.href(), "/home");
        assert_eq!(NavRoute::Words.href(), "/words");
        assert_eq!(NavRoute::Grammar.href(), "/grammar");
        assert_eq!(NavRoute::Kanji.href(), "/kanji");
        assert_eq!(NavRoute::Phrases.href(), "/phrases");
        assert_eq!(NavRoute::Profile.href(), "/profile");
    }

    #[test]
    fn is_active_root_paths_map_to_home() {
        assert!(NavRoute::Home.is_active("/"));
        assert!(NavRoute::Home.is_active(""));
        assert!(NavRoute::Home.is_active("/home"));
        assert!(!NavRoute::Words.is_active("/home"));
    }

    #[test]
    fn is_active_sets_route_covers_sets() {
        assert!(NavRoute::Words.is_active("/words"));
        assert!(NavRoute::Words.is_active("/sets"));
        assert!(NavRoute::Words.is_active("/sets/some-id"));
        assert!(!NavRoute::Grammar.is_active("/sets"));
    }

    #[test]
    fn is_active_detail_paths_match_their_section() {
        assert!(NavRoute::Grammar.is_active("/grammar/123"));
        assert!(NavRoute::Kanji.is_active("/kanji/食"));
        assert!(NavRoute::Phrases.is_active("/phrases"));
        assert!(NavRoute::Profile.is_active("/profile"));
    }

    #[test]
    fn all_contains_six_routes_sidebar_excludes_profile() {
        assert_eq!(NavRoute::all().len(), 6);
        assert!(!NavRoute::sidebar_routes().contains(&NavRoute::Profile));
        assert_eq!(NavRoute::sidebar_routes().len(), 5);
    }

    #[test]
    fn only_home_uses_logo() {
        assert!(NavRoute::Home.use_logo());
        assert!(!NavRoute::Words.use_logo());
        assert!(!NavRoute::Profile.use_logo());
    }

    #[test]
    fn test_id_suffix_unique_per_route() {
        let suffixes: Vec<_> = NavRoute::all().iter().map(|r| r.test_id_suffix()).collect();
        let mut unique = suffixes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(suffixes.len(), unique.len(), "suffixes must be unique");
    }
}
