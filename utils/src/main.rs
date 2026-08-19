mod api;
mod cli;
mod commands;
mod dictionary;
mod utils;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::commands::{
    run_build_phrase_dataset, run_dedup_kanji_readings, run_enrich_phrases_with_grammar,
    run_find_missing, run_generate_grammar, run_generate_grammar_prompt, run_ndlocr,
    run_patch_kanji_readings, run_regenerate_invalid, run_tokenize, run_tokenize_well_known,
    run_validate_dictionary,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Tokenize { text, file } => run_tokenize(text, file),
        Commands::Ndlocr {
            input,
            detector,
            rec30,
            rec50,
            rec100,
            vocab,
        } => run_ndlocr(input, detector, rec30, rec50, rec100, vocab),
        Commands::TokenizeWellKnown { path } => run_tokenize_well_known(path),
        Commands::FindMissing {
            output,
            generate,
            api_base,
            api_key,
            model,
            workers,
            chunk_size,
            russian_only,
            english_only,
        } => {
            run_find_missing(
                output,
                generate,
                api_base,
                api_key,
                model,
                workers,
                chunk_size,
                russian_only,
                english_only,
            )
            .await
        },
        Commands::BuildPhraseDataset {
            input,
            output,
            min_tokens,
        } => run_build_phrase_dataset(input, output, min_tokens),
        Commands::ValidateDictionary {
            api_key,
            api_base,
            model,
            workers,
            output,
            dry_run,
            limit,
        } => {
            run_validate_dictionary(api_key, api_base, model, workers, output, dry_run, limit).await
        },
        Commands::GenerateGrammarPrompt {
            title,
            level,
            rule_name_from_index,
        } => run_generate_grammar_prompt(title, level, rule_name_from_index),
        Commands::GenerateGrammar {
            rule_id,
            all,
            indices,
            level,
            api_base,
            api_key,
            workers,
            dry_run,
            grammar_path,
            model,
            reasoning,
        } => {
            run_generate_grammar(
                rule_id,
                all,
                indices,
                level,
                api_base,
                api_key,
                workers,
                dry_run,
                grammar_path,
                model,
                reasoning,
            )
            .await
        },
        Commands::RegenerateInvalid {
            input,
            api_base,
            api_key,
            workers,
            dry_run,
            russian_only,
            english_only,
        } => {
            run_regenerate_invalid(
                input,
                api_base,
                api_key,
                workers,
                dry_run,
                russian_only,
                english_only,
            )
            .await
        },
        Commands::DedupKanjiReadings { input, dry_run } => run_dedup_kanji_readings(input, dry_run),
        Commands::PatchKanjiReadings {
            input,
            patches,
            dry_run,
        } => run_patch_kanji_readings(input, patches, dry_run),
        Commands::RetokenizePhrases { index, data_dir } => {
            commands::retokenize_phrases::run_retokenize_phrases(index, data_dir)
        },
        Commands::EnrichPhrasesWithGrammar {
            input,
            chunks_dir,
            grammar,
            output,
            dictionary_dir,
        } => run_enrich_phrases_with_grammar(input, chunks_dir, grammar, output, dictionary_dir),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
