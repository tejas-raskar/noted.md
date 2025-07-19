mod ai_provider;
mod cli;
mod clients;
mod config;
mod error;
mod file_utils;
mod notion;
mod ui;

use ai_provider::AiProvider;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use config::{ClaudeConfig, Config, GeminiConfig, OllamaConfig};
use dialoguer::Input;
use dialoguer::Select;
use dialoguer::{Password, theme::ColorfulTheme};
use error::NotedError;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use crate::clients::claude_client::ClaudeClient;
use crate::clients::gemini_client::GeminiClient;
use crate::clients::ollama_client::OllamaClient;
use crate::clients::openai_client::OpenAIClient;
use crate::config::OpenAIConfig;
use crate::notion::converter::Converter;
use std::path::Path;
use ui::{ascii_art, print_clean_config};

use crate::config::get_config_path;

fn test_parser() {
    let markdown = "# Bullet list\n\n- Item 1\n- Item 2\n\n## Numbered List\n\n1. First item\n2. Second item\n\n## Heading 2\n\nSome text here.";
    let arena = comrak::Arena::new();
    let parsed = Converter::run(markdown, &arena);
    println!("{:#?}", parsed.unwrap());
}

async fn process_and_save_file(
    file_path: &str,
    client: &dyn AiProvider,
    output_dir: Option<&str>,
    progress_bar: &ProgressBar,
) -> Result<(), NotedError> {
    let path = Path::new(file_path);
    let file_name = match path.file_name() {
        Some(name) => name,
        None => {
            return Err(NotedError::FileNameError(file_path.to_string()));
        }
    };

    progress_bar.println(format!(
        "\n{}",
        format!("Processing file: {:#?}", file_name).bold()
    ));

    let file_data = file_utils::process_file(file_path)?;
    progress_bar.println(format!(
        "{} {}",
        "✔".green(),
        "File read successfully.".green()
    ));

    progress_bar.set_message(format!("{}", "Sending to your AI model...".yellow()));

    let markdown = client.send_request(file_data).await?;
    progress_bar.println(format!("{} {}", "✔".green(), "Received response.".green()));

    let output_path = match output_dir {
        Some(dir) => {
            let dir_path = Path::new(dir);
            if !dir_path.exists() {
                std::fs::create_dir_all(dir_path)?;
            }
            let final_path = dir_path.join(file_name);
            final_path
                .with_extension("md")
                .to_string_lossy()
                .into_owned()
        }
        None => path.with_extension("md").to_string_lossy().into_owned(),
    };

    match std::fs::write(&output_path, markdown) {
        Ok(_) => {
            progress_bar.println(format!(
                "{} {}",
                "✔".green(),
                format!("Markdown saved to '{}'", output_path.cyan()).green()
            ));
            Ok(())
        }
        Err(e) => {
            progress_bar.println(format!(
                "{} {}",
                "✖".red(),
                format!("Failed to save file to '{}'. Error: {}", &output_path, e).red()
            ));
            Err(e.into())
        }
    }
}

async fn run() -> Result<(), NotedError> {
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
                        return Err(NotedError::ConfigNotFound);
                    }
                }
            }

            if show {
                if let Some(config_path) = config::get_config_path() {
                    if config_path.exists() {
                        let config = Config::load()?;
                        print_clean_config(config);
                    } else {
                        return Err(NotedError::ConfigNotFound);
                    }
                }
            }

            if let Some(ref key) = set_api_key {
                let mut config = Config::load()?;
                config.active_provider = Some("gemini".to_string());
                config.gemini = Some(config::GeminiConfig {
                    api_key: key.to_string(),
                });

                config.save()?;
                println!("Config saved successfully.");
            }

            if let Some(ref key) = set_claude_api_key {
                let mut config = Config::load()?;
                config.active_provider = Some("claude".to_string());
                let model = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Claude model")
                    .default("claude-3-opus-20240229".to_string())
                    .interact_text()?;

                config.claude = Some(config::ClaudeConfig {
                    api_key: key.to_string(),
                    model,
                });

                config.save()?;
                println!("Config saved successfully.");
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
                    "OpenAI Compatible API (Cloud/Local, works with LM Studio)",
                ];
                let selected_provider = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Choose your AI provider")
                    .items(&providers)
                    .default(0)
                    .interact()?;

                match selected_provider {
                    0 => {
                        let mut config = Config::load()?;
                        let api_key = Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter your Gemini API key: ")
                            .interact()?;
                        config.active_provider = Some("gemini".to_string());
                        config.gemini = Some(GeminiConfig { api_key });
                        config.save()?;
                        println!("{}", "Config saved successfully.".green());
                    }
                    1 => {
                        let mut config = Config::load()?;
                        let api_key = Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter your Claude API key: ")
                            .interact()?;
                        config.active_provider = Some("claude".to_string());
                        let anthropic_models = vec![
                            "    claude-opus-4-20250514",
                            "    claude-sonnet-4-20250514",
                            "    claude-3-7-sonnet-20250219",
                            "    claude-3-5-haiku-20241022",
                            "    claude-3-5-sonnet-20241022",
                            "    Other",
                        ];
                        let selected_model = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Choose your Claude model:")
                            .items(&anthropic_models)
                            .default(0)
                            .interact()?;

                        let model = if selected_model == anthropic_models.len() - 1 {
                            Input::with_theme(&ColorfulTheme::default())
                                .with_prompt("Enter the custom model name:")
                                .interact_text()?
                        } else {
                            anthropic_models[selected_model].trim().to_string()
                        };

                        config.claude = Some(ClaudeConfig { api_key, model });
                        config.save()?;
                        println!("{}", "Config saved successfully.".green());
                    }
                    2 => {
                        let url = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Ollama server url")
                            .default("http://localhost:11434".to_string())
                            .interact_text()?;

                        let model = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Ollama model")
                            .default("gemma3:27b".to_string())
                            .interact_text()?;

                        let mut config = Config::load()?;
                        config.active_provider = Some("ollama".to_string());
                        config.ollama = Some(OllamaConfig { url, model });
                        config.save()?;
                        println!("{}", "Config saved successfully.".green());
                    }
                    3 => {
                        let url = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Server url")
                            .default("http://localhost:1234".to_string())
                            .interact_text()?;

                        let model = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Model")
                            .default("gemma3:27b".to_string())
                            .interact_text()?;

                        let api_key_str = Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter your API key (Optional, press Enter if none): ")
                            .allow_empty_password(true)
                            .interact()?;

                        let api_key = if api_key_str.is_empty() {
                            None
                        } else {
                            Some(api_key_str)
                        };

                        let mut config = Config::load()?;
                        config.active_provider = Some("openai".to_string());
                        config.openai = Some(OpenAIConfig {
                            url,
                            model,
                            api_key,
                        });
                        config.save()?;
                        println!("{}", "Config saved successfully.".green());
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
                        return Err(NotedError::ConfigNotFound);
                    }

                    let mut config = Config::load()?;
                    let new_provider_str = new_provider.as_str();
                    let is_configured = match new_provider_str {
                        "gemini" => config.gemini.is_some(),
                        "claude" => config.claude.is_some(),
                        "ollama" => config.ollama.is_some(),
                        "openai" => config.openai.is_some(),
                        _ => {
                            eprintln!(
                                "Invalid provider '{}'. Please choose from 'gemini', 'claude', or 'ollama'.",
                                new_provider
                            );
                            return Ok(());
                        }
                    };

                    if is_configured {
                        config.active_provider = Some(new_provider_str.to_string());
                        config.save()?;
                        println!("Active provider set to '{}'.", new_provider_str.cyan());
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
                        let config = Config::load()?;
                        print_clean_config(config);
                    } else {
                        return Err(NotedError::ConfigNotFound);
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
            let config = Config::load()?;
            let client: Box<dyn AiProvider> = match config.active_provider.as_deref() {
                Some("gemini") => {
                    let final_api_key = if let Some(key) = api_key {
                        key
                    } else if let Some(gemini_config) = &config.gemini {
                        gemini_config.api_key.clone()
                    } else {
                        return Err(NotedError::GeminiNotConfigured);
                    };
                    Box::new(GeminiClient::new(final_api_key, prompt))
                }
                Some("ollama") => {
                    let url = if let Some(ollama_config) = &config.ollama {
                        ollama_config.url.clone()
                    } else {
                        return Err(NotedError::OllamaNotConfigured);
                    };
                    let model = if let Some(ollama_config) = &config.ollama {
                        ollama_config.model.clone()
                    } else {
                        return Err(NotedError::OllamaNotConfigured);
                    };
                    Box::new(OllamaClient::new(url, model, prompt))
                }
                Some("claude") => {
                    let api_key = if let Some(key) = api_key {
                        key
                    } else if let Some(claude_config) = &config.claude {
                        claude_config.api_key.clone()
                    } else {
                        return Err(NotedError::ClaudeNotConfigured);
                    };

                    let model = if let Some(claude_config) = &config.claude {
                        claude_config.model.clone()
                    } else {
                        return Err(NotedError::ClaudeNotConfigured);
                    };

                    Box::new(ClaudeClient::new(api_key, model, prompt))
                }
                Some("openai") => {
                    let url = if let Some(openai_config) = &config.openai {
                        openai_config.url.clone()
                    } else {
                        return Err(NotedError::OpenAINotConfigured);
                    };
                    let model = if let Some(openai_config) = &config.openai {
                        openai_config.model.clone()
                    } else {
                        return Err(NotedError::OpenAINotConfigured);
                    };
                    let api_key = if let Some(openai_config) = &config.openai {
                        openai_config.api_key.clone()
                    } else {
                        return Err(NotedError::OpenAINotConfigured);
                    };
                    Box::new(OpenAIClient::new(url, model, api_key, prompt))
                }
                _ => return Err(NotedError::NoActiveProvider),
            };

            let input_path = Path::new(&path);
            if !input_path.exists() {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Input path not found: {}", path),
                )));
            }

            if input_path.is_dir() {
                let files_to_convert: Vec<_> = std::fs::read_dir(input_path)?
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(path_str) = path.to_str() {
                                if file_utils::get_file_mime_type(path_str).is_ok() {
                                    return Some(path);
                                }
                            }
                        }
                        None
                    })
                    .collect();

                if files_to_convert.is_empty() {
                    println!("No supported files found in the directory.");
                    return Ok(());
                }

                let progress_bar = ProgressBar::new(files_to_convert.len() as u64);
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                        .unwrap(),
                );
                progress_bar.set_message("Processing files...");

                for file_path_buf in files_to_convert {
                    if let Some(file_path_str) = file_path_buf.to_str() {
                        if let Err(e) = process_and_save_file(
                            file_path_str,
                            client.as_ref(),
                            output.as_deref(),
                            &progress_bar,
                        )
                        .await
                        {
                            progress_bar.println(format!("{}", e.to_string().red()));
                        }
                    }
                    progress_bar.inc(1);
                }

                progress_bar
                    .finish_with_message(format!("{}", "Completed processing all files".green()));
            } else {
                let path_str = input_path.to_str().ok_or_else(|| {
                    NotedError::FileNameError(input_path.to_string_lossy().to_string())
                })?;
                file_utils::get_file_mime_type(path_str)?;
                let progress_bar = ProgressBar::new(1);
                progress_bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                        .unwrap(),
                );
                progress_bar.set_message("Processing file...");
                if let Err(e) = process_and_save_file(
                    path_str,
                    client.as_ref(),
                    output.as_deref(),
                    &progress_bar,
                )
                .await
                {
                    progress_bar.println(format!("{}", e.to_string().red()));
                }
                progress_bar.inc(1);
                progress_bar
                    .finish_with_message(format!("{}", "Completed processing file".green()));
            }
        }
    }
    Ok(())
}
#[tokio::main]
async fn main() {
    test_parser();
    // if let Err(e) = run().await {
    //     eprintln!("{} {}", "✖".red(), e.to_string().red());
    //     std::process::exit(1);
    // }
}
