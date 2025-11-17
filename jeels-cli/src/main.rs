use jeels_cli::{application::LlmService, infrastructure::QwenLlm, settings::Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load().await?;

    println!("[LOG] Начало загрузки модели...");
    let load_start = std::time::Instant::now();
    let llm = QwenLlm::new(&settings.llm)?;
    let load_duration = load_start.elapsed();
    println!(
        "[LOG] Модель загружена за {:.2} секунд",
        load_duration.as_secs_f64()
    );

    println!("[LOG] Начало генерации ответа...");
    let gen_start = std::time::Instant::now();
    let answer = LlmService::generate_answer(
        &llm,
        "Переведи на русский: '食べます'. Отвечай кратко, но емко",
    )
    .await?;
    let gen_duration = gen_start.elapsed();
    println!(
        "[LOG] Ответ сгенерирован за {:.2} секунд",
        gen_duration.as_secs_f64()
    );
    println!("{}", answer);

    Settings::init(settings)?;
    jeels_cli::cli::run_cli().await?;
    Ok(())
}
