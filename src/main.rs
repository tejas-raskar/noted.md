mod cli;
mod config;
mod file_utils;
mod gemini_client;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use gemini_client::GeminiClient;
use std::path::Path;

async fn process_and_save_file(file_path: &str, client: &GeminiClient, output_dir: Option<&str>) {
    let path = Path::new(file_path);
    let file_name = match path.file_name() {
        Some(name) => name,
        None => {
            eprintln!("Could not determine filename for {}. Skipping.", file_path);
            return;
        }
    };

    println!("Processing file: {:#?}", file_name);
    let file_data = match file_utils::process_file(file_path) {
        Ok(data) => {
            println!("{:#?} processed successfully.", file_name);
            data
        }
        Err(e) => {
            eprintln!(
                "Failed to process {:#?}. Skipping file. Error: {}",
                file_name, e
            );
            return;
        }
    };

    println!(">>> Sending request to Gemini");
    let response = match client.send_request(file_data).await {
        Ok(res) => {
            println!(">>> Successfully received response from Gemini");
            res
        }
        Err(e) => {
            eprintln!("Error calling Gemini: {}", e);
            return;
        }
    };

    let response_text = response
        .candidates
        .get(0)
        .and_then(|candidate| candidate.content.parts.get(0));
    let markdown_text = if let Some(part) = response_text {
        &part.text
    } else {
        eprintln!(
            ">>> Could not find text in Gemini response for {:?}. Skipping.",
            file_name
        );
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
        Ok(_) => println!(">>> Successfully saved markdown to {}", output_path),
        Err(e) => {
            eprintln!(">>> Error saving the markdown file: {}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

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
                let config = Config::load();
                if let Some(gemini_config) = config.gemini {
                    gemini_config.api_key
                } else {
                    eprintln!(">>> Error: API key not provided.");
                    eprintln!(
                        "Please set it with `notedmd config --set-api-key <YOUR_KEY>` or use the --api-key flag."
                    );
                    std::process::exit(1);
                }
            };

            let client = GeminiClient::new(final_api_key);
            let input_path = Path::new(&path);
            if input_path.is_dir() {
                for entry in std::fs::read_dir(input_path).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let mime_type = file_utils::get_file_mime_type(path.to_str().unwrap());
                    if path.is_file() && mime_type != "application/octet-stream" {
                        process_and_save_file(path.to_str().unwrap(), &client, output.as_deref())
                            .await;
                    }
                }
            } else {
                process_and_save_file(&path, &client, output.as_deref()).await;
            }
        }
    }
}
