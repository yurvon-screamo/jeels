use keikaku::cli::run_cli;
use keikaku::settings::ApplicationEnvironment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    run_cli().await?;
    Ok(())
}
