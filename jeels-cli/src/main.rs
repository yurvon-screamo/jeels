use jeels_cli::settings::ApplicationEnvironment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    jeels_cli::cli::run_cli().await?;
    Ok(())
}
