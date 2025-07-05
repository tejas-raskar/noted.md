mod ai_provider;
mod cli;
mod clients;
mod config;
mod file_utils;
mod ui;

use ai_provider::AiProvider;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use config::{ClaudeConfig, Config, GeminiConfig, OllamaConfig};
use dialoguer::Input;
use dialoguer::Select;
use dialoguer::{Password, theme::ColorfulTheme};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use crate::clients::claude_client::ClaudeClient;
use crate::clients::gemini_client::GeminiClient;
use crate::clients::ollama_client::OllamaClient;
use std::path::Path;
use ui::{ascii_art, print_clean_config};

use crate::config::get_config_path;

async fn process_and_save_file(
    file_path: &str,
    client: &dyn AiProvider,
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

    progress_bar.set_message(format!("{}", "Sending to your AI model...".yellow()));

    let markdown = match client.send_request(file_data).await {
        Ok(cleaned_markdown) => cleaned_markdown,
        Err(e) => {
            progress_bar.println(format!("{} {}", "✖".red(), format!("Error: {}", e).red()));
            return;
        }
    };
    progress_bar.println(format!("{} {}", "✔".green(), "Received response.".green()));

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

    match std::fs::write(&output_path, markdown) {
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
    match args.command {
        Commands::Config {
            set_api_key,
            set_claude_api_key,
            set_provider,
            show_path,
            show,
            edit,
        } => {
            if show_path {
                if let Some(config_path) = config::get_config_path() {
                    if config_path.exists() {
                        println!("Config saved in {:?}", config_path);
                    } else {
                        eprintln!("No config found.\nRun 'notedmd config --edit' to configure.");
                        return;
                    }
                }
            }

            if show {
                if let Some(config_path) = config::get_config_path() {
                    if config_path.exists() {
                        let config = Config::load();
                        print_clean_config(config);
                    } else {
                        eprintln!("No config found.\nRun 'notedmd config --edit' to configure.");
                        return;
                    }
                }
            }

            if let Some(ref key) = set_api_key {
                let mut config = Config::load();
                config.active_provider = Some("gemini".to_string());
                config.gemini = Some(config::GeminiConfig {
                    api_key: key.to_string(),
                });

                if let Err(e) = config.save() {
                    println!("Error while saving config: {}", e)
                } else {
                    println!("Config saved successfully.");
                }
            }

            if let Some(ref key) = set_claude_api_key {
                let mut config = Config::load();
                config.active_provider = Some("claude".to_string());
                let model = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Claude model")
                    .default("claude-3-opus-20240229".to_string())
                    .interact_text()
                    .unwrap();

                config.claude = Some(config::ClaudeConfig {
                    api_key: key.to_string(),
                    model: model,
                });

                if let Err(e) = config.save() {
                    println!("Error while saving config: {}", e)
                } else {
                    println!("Config saved successfully.");
                }
            }

            if edit {
                ascii_art();
                println!(
                    "{}\n",
                    "Welcome to noted.md! Let's set up your AI provider.".bold()
                );

                let providers = vec![
                    "Gemini API (Cloud-based, requires API key)",
                    "Claude API (Cloud-based, requires API key)",
                    "Ollama (Local, requires Ollama to be set up)",
                ];
                let selected_provider = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Choose your AI provider")
                    .items(&providers)
                    .default(0)
                    .interact()
                    .unwrap();

                match selected_provider {
                    0 => {
                        let mut config = Config::load();
                        let _api_key = match Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter your Gemini API key: ")
                            .interact()
                        {
                            Ok(key) => {
                                config.active_provider = Some("gemini".to_string());
                                config.gemini = Some(GeminiConfig {
                                    api_key: key.clone(),
                                });
                                let _save = match config.save() {
                                    Ok(()) => {
                                        println!("{}", "Config saved successfully.".green())
                                    }
                                    Err(e) => {
                                        eprintln!("{}:{}", "Error saving the config.".red(), e)
                                    }
                                };
                                key
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(1);
                            }
                        };
                    }
                    1 => {
                        let mut config = Config::load();
                        let _api_key = match Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter your Claude API key: ")
                            .interact()
                        {
                            Ok(key) => {
                                config.active_provider = Some("claude".to_string());
                                let anthropic_models = vec![
                                    "    claude-opus-4-20250514",
                                    "    claude-sonnet-4-20250514",
                                    "    claude-3-7-sonnet-20250219",
                                    "    claude-3-5-haiku-20241022",
                                    "    claude-3-5-sonnet-20241022",
                                    "    other",
                                ];
                                let selected_model = Select::with_theme(&ColorfulTheme::default())
                                    .with_prompt("Choose your Claude model:")
                                    .items(&anthropic_models)
                                    .default(0)
                                    .interact()
                                    .unwrap();

                                if selected_model == &anthropic_models.len() - 1 {
                                    let other_model =
                                        Input::with_theme(&ColorfulTheme::default())
                                            .with_prompt(
                                                "Enter the custom model name (e.g., claude-3-sonnet-20240229)",
                                            )
                                            .interact()
                                            .unwrap();

                                    config.claude = Some(ClaudeConfig {
                                        api_key: key.clone(),
                                        model: other_model,
                                    });
                                } else {
                                    config.claude = Some(ClaudeConfig {
                                        api_key: key.clone(),
                                        model: anthropic_models[selected_model].trim().to_string(),
                                    });
                                }

                                let _save = match config.save() {
                                    Ok(()) => {
                                        println!("{}", "Config saved successfully.".green())
                                    }
                                    Err(e) => {
                                        eprintln!("{}:{}", "Error saving the config.".red(), e)
                                    }
                                };
                                key
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(1);
                            }
                        };
                    }
                    2 => {
                        let url = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Ollama server url")
                            .default("http://localhost:11434".to_string())
                            .interact_text()
                            .unwrap();

                        let model = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Ollama model")
                            .default("gemma3:27b".to_string())
                            .interact_text()
                            .unwrap();

                        let mut config = Config::default();
                        config.active_provider = Some("ollama".to_string());
                        config.ollama = Some(OllamaConfig {
                            url: url,
                            model: model,
                        });
                        let _save = match config.save() {
                            Ok(()) => {
                                println!("{}", "Config saved successfully.".green())
                            }
                            Err(e) => {
                                eprintln!("{}:{}", "Error saving the config.".red(), e)
                            }
                        };
                    }
                    _ => unreachable!(),
                }
                println!(
                    "{}",
                    "You can now run 'notedmd convert <file>' to convert your files.".cyan()
                );
            }

            if let Some(ref new_provider) = set_provider {
                if let Some(config_path) = get_config_path() {
                    if !config_path.exists() {
                        eprintln!(
                            "No configuration file found. Please run 'notedmd config --edit' to get started."
                        );
                        return;
                    }

                    let mut config = Config::load();
                    let new_provider_str = new_provider.as_str();
                    let is_configured = match new_provider_str {
                        "gemini" => config.gemini.is_some(),
                        "claude" => config.claude.is_some(),
                        "ollama" => config.ollama.is_some(),
                        _ => {
                            eprintln!(
                                "Invalid provider '{}'. Please choose from 'gemini', 'claude', or 'ollama'.",
                                new_provider
                            );
                            return;
                        }
                    };

                    if is_configured {
                        config.active_provider = Some(new_provider_str.to_string());
                        if let Err(e) = config.save() {
                            eprintln!("{}: {}", "Error saving the config.".red(), e);
                        } else {
                            println!("Active provider set to '{}'.", new_provider_str.cyan());
                        }
                    } else {
                        eprintln!(
                            "{} is not configured. Please run 'notedmd config --edit' to set it up.",
                            new_provider_str.yellow()
                        );
                    }
                }
            }

            if !edit
                && !show
                && !show_path
                && set_api_key.is_none()
                && set_claude_api_key.is_none()
                && set_provider.is_none()
            {
                if let Some(config_path) = get_config_path() {
                    if config_path.exists() {
                        let config = Config::load();
                        print_clean_config(config);
                    } else {
                        eprintln!("No config found.\nRun 'notedmd config --edit' to configure.");
                        return;
                    }
                }
            }
        }
        Commands::Convert {
            path,
            output,
            api_key,
            prompt,
        } => {
            let config = Config::load();
            let client: Box<dyn AiProvider> = match config.active_provider.as_deref() {
                Some("gemini") => {
                    let final_api_key = if let Some(key) = api_key {
                        key
                    } else if let Some(gemini_config) = &config.gemini {
                        gemini_config.api_key.clone()
                    } else {
                        eprintln!("{}", "Gemini is not configured properly. Run 'notedmd config' to configure it.".red());
                        std::process::exit(1);
                    };
                    Box::new(GeminiClient::new(final_api_key, prompt))
                }
                Some("ollama") => {
                    let url = if let Some(ollama_config) = &config.ollama {
                        ollama_config.url.clone()
                    } else {
                        eprintln!("{}", "Ollama is not configured properly. Run 'notedmd config' to configure it.".red());
                        std::process::exit(1);
                    };
                    let model = if let Some(ollama_config) = &config.ollama {
                        ollama_config.model.clone()
                    } else {
                        eprintln!("{}", "Ollama is not configured properly. Run 'notedmd config' to configure it.".red());
                        std::process::exit(1);
                    };
                    Box::new(OllamaClient::new(url, model, prompt))
                }
                Some("claude") => {
                    let api_key = if let Some(key) = api_key {
                        key
                    } else if let Some(claude_config) = &config.claude {
                        claude_config.api_key.clone()
                    } else {
                        eprintln!("{}", "Claude is not configured properly. Run 'notedmd config' to configure it.".red());
                        std::process::exit(1);
                    };

                    let model = if let Some(claude_config) = &config.claude {
                        claude_config.model.clone()
                    } else {
                        eprintln!("{}", "Claude is not configured properly. Run 'notedmd config' to configure it.".red());
                        std::process::exit(1);
                    };

                    Box::new(ClaudeClient::new(api_key, model, prompt))
                }
                _ => {
                    eprintln!(
                        "{}",
                        "notedmd is not configured. Run 'notedmd config' to configure it first."
                            .red()
                    );
                    std::process::exit(1);
                }
            };

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
                        client.as_ref(),
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
                process_and_save_file(&path, client.as_ref(), output.as_deref(), &progress_bar)
                    .await;
                progress_bar.inc(1);
                progress_bar
                    .finish_with_message(format!("{}", "Completed processing file".green()));
            }
        }
    }
}
