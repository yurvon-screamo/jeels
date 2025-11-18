use jeels_cli::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Settings::load().await?;
    jeels_cli::cli::run_cli().await?;
    Ok(())
}
