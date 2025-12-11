use keikaku::settings::ApplicationEnvironment;
use keikaku_cli::cli::run_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    run_cli().await?;
    Ok(())
}
