mod cli;
mod config;
mod file_utils;
mod gemini_client;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use config::Config;
use config::GeminiConfig;
use dialoguer::{Password, theme::ColorfulTheme};
use gemini_client::GeminiClient;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::path::Path;
use ui::ascii_art;

async fn process_and_save_file(
    file_path: &str,
    client: &GeminiClient,
    output_dir: Option<&str>,
    progress_bar: &ProgressBar,
) {
    let path = Path::new(file_path);
    let file_name = match path.file_name() {
        Some(name) => name,
        None => {
            progress_bar.println(format!(
                "{} {}",
                "✖".red(),
                format!(
                    "Could not determine filename for '{}'. Skipping.",
                    file_path
                )
                .red()
            ));
            return;
        }
    };

    progress_bar.println(format!(
        "\n{}",
        format!("Processing file: {:#?}", file_name).bold()
    ));

    let file_data = match file_utils::process_file(file_path) {
        Ok(data) => data,
        Err(e) => {
            progress_bar.println(format!(
                "{} {}",
                "✖".red(),
                format!("Failed to read file. Error: {}", e).red()
            ));
            return;
        }
    };
    progress_bar.println(format!(
        "{} {}",
        "✔".green(),
        "File read successfully.".green()
    ));

    progress_bar.set_message(format!("{}", "Sending to Gemini...".yellow()));

    let response = match client.send_request(file_data).await {
        Ok(res) => res,
        Err(e) => {
            progress_bar.println(format!(
                "{} {}",
                "✖".red(),
                format!("Error calling Gemini: {}", e).red()
            ));
            return;
        }
    };
    progress_bar.println(format!(
        "{} {}",
        "✔".green(),
        "Received response from Gemini".green()
    ));

    let response_text = response
        .candidates
        .first()
        .and_then(|candidate| candidate.content.parts.first());
    let markdown_text = if let Some(part) = response_text {
        &part.text
    } else {
        progress_bar.println(format!(
            "Could not find text in Gemini response for {:?}. Skipping.",
            file_name
        ));
        return;
    };

    let cleaned_markdown = markdown_text
        .trim_start_matches("```markdown\n")
        .trim_end_matches("```");

    let output_path = match output_dir {
        Some(dir) => {
            let dir_path = Path::new(dir);
            let final_path = dir_path.join(file_name);
            final_path
                .with_extension("md")
                .to_string_lossy()
                .into_owned()
        }
        None => path.with_extension("md").to_string_lossy().into_owned(),
    };

    match std::fs::write(&output_path, cleaned_markdown) {
        Ok(_) => progress_bar.println(format!(
            "{} {}",
            "✔".green(),
            format!("Markdown saved to '{}'", output_path.cyan()).green()
        )),
        Err(e) => progress_bar.println(format!(
            "{} {}",
            "✖".red(),
            format!("Failed to save file. Error: {}", e).red()
        )),
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    ascii_art();
    match args.command {
        Commands::Config {
            set_api_key,
            show_path,
        } => {
            if show_path {
                if let Some(config_path) = config::get_config_path() {
                    if config_path.exists() {
                        println!(">>> {:?}", config_path);
                    } else {
                        eprintln!(">>> No config path found.");
                        return;
                    }
                }
            }

            if let Some(key) = set_api_key {
                let mut config = Config::load();
                config.gemini = Some(config::GeminiConfig { api_key: key });

                if let Err(e) = config.save() {
                    println!(">>> Error while saving config: {}", e)
                } else {
                    println!(">>> Config saved successfully.");
                }
            }
        }
        Commands::Convert {
            path,
            output,
            api_key,
        } => {
            let final_api_key = if let Some(key) = api_key {
                key
            } else {
                let mut config = Config::load();
                if let Some(gemini_config) = config.gemini {
                    gemini_config.api_key
                } else {
                    ascii_art();
                    let api_key = match Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter your Gemini API key: ")
                        .interact()
                    {
                        Ok(key) => {
                            config.gemini = Some(GeminiConfig {
                                api_key: key.clone(),
                            });
                            let _save = match config.save() {
                                Ok(()) => println!("{}", "Config saved successfully.".green()),
                                Err(e) => eprintln!("{}:{}", "Error saving the config.".red(), e),
                            };
                            key
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        }
                    };

                    api_key
                }
            };

            let client = GeminiClient::new(final_api_key);
            let input_path = Path::new(&path);
            if input_path.is_dir() {
                let files_to_convert: Vec<_> = std::fs::read_dir(input_path)
                    .unwrap()
                    .filter_map(|entry| {
                        let entry = entry.ok()?;
                        let path = entry.path();
                        let mime_type = file_utils::get_file_mime_type(path.to_str()?);
                        if path.is_file() && mime_type != "application/octet-stream" {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect();

                let progress_bar = ProgressBar::new(files_to_convert.len() as u64);
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                        .unwrap(),
                );

                progress_bar.set_message("Processing files...");

                for file_path_buf in files_to_convert {
                    process_and_save_file(
                        file_path_buf.to_str().unwrap(),
                        &client,
                        output.as_deref(),
                        &progress_bar,
                    )
                    .await;
                    progress_bar.inc(1);
                }

                progress_bar
                    .finish_with_message(format!("{}", "Completed processing all files".green()));
            } else {
                let progress_bar = ProgressBar::new(1);
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                        .unwrap(),
                );
                progress_bar.set_message("Processing file...");
                process_and_save_file(&path, &client, output.as_deref(), &progress_bar).await;
                progress_bar.inc(1);
                progress_bar
                    .finish_with_message(format!("{}", "Completed processing file".green()));
            }
        }
    }
}
