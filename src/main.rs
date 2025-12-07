use keikaku::settings::ApplicationEnvironment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    keikaku::cli::run_cli().await?;
    Ok(())
}
