use jeels_cli::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load().await?;
    Settings::init(settings)?;
    jeels_cli::cli::run_cli().await?;
    Ok(())
}
