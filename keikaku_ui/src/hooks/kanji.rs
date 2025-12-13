use dioxus::prelude::*;
use keikaku::application::use_cases::get_kanji_info::GetKanjiInfoUseCase;
use keikaku::domain::dictionary::KanjiInfo;

pub fn use_kanji() -> UseKanji {
    use_hook(|| UseKanji {
        query: use_signal(|| "語".to_string()),
        info: use_signal(|| None as Option<KanjiInfo>),
        loading: use_signal(|| false),
    })
}

#[derive(Clone, PartialEq)]
pub struct UseKanji {
    pub query: Signal<String>,
    pub info: Signal<Option<KanjiInfo>>,
    pub loading: Signal<bool>,
}

impl UseKanji {
    pub fn fetch_kanji_info(&mut self) {
        let query = (self.query)();
        if query.trim().is_empty() {
            return;
        }

        self.loading.set(true);
        let mut info = self.info;

        spawn(async move {
            match GetKanjiInfoUseCase::new().execute(&query) {
                Ok(kanji_info) => info.set(Some(kanji_info)),
                Err(_) => info.set(None),
            }
        });
    }
}
