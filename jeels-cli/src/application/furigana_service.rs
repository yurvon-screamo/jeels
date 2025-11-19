pub trait FuriganaService: Send + Sync {
    fn get_furigana(&self, text: &str) -> String;
}
