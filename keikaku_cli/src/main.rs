use keikaku::settings::ApplicationEnvironment;
use keikaku_cli::cli::run_cli;

mod anki;
mod card;
mod cli;
mod duolingo;
mod furigana_renderer;
mod jlpt;
mod kanji;
mod learn;
mod me;
mod migii;
mod translate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    run_cli().await?;
    Ok(())
}
