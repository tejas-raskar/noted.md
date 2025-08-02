mod ai_provider;
mod cli;
mod clients;
mod config;
mod error;
mod file_utils;
mod pdf_utils;
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
use std::{fs, path::Path, collections::BTreeSet};
use ui::{ascii_art, print_clean_config};

use crate::config::get_config_path;
use crate::pdf_utils::{ProgressTracker, ProcessingProgress, process_pdf, extract_page_as_image};

// Helper function to parse page ranges
fn parse_page_ranges(
    page_selection: &str,
    total_pages: u32,
) -> Result<Vec<u32>, NotedError> {
    let mut pages = BTreeSet::new(); // Use BTreeSet for sorted unique pages

    for part in page_selection.split(',') {
        if part.contains('-') {
            let mut range_parts = part.split('-');
            let start = range_parts.next().ok_or_else(|| {
                NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Malformed page range: {}", part),
                ))
            })?.trim().parse::<u32>().map_err(|_| {
                NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid start page in range: {}", part),
                ))
            })?;
            let end = range_parts.next().ok_or_else(|| {
                NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Malformed page range: {}", part),
                ))
            })?.trim().parse::<u32>().map_err(|_| {
                NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid end page in range: {}", part),
                ))
            })?;

            if start == 0 || end == 0 {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Page numbers must be 1 or greater.".to_string(),
                )));
            }

            if start > end {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Start page cannot be greater than end page in range: {}", part),
                )));
            }
            
            for page_num in start..=end {
                if page_num > total_pages {
                    return Err(NotedError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Page {} in range {} exceeds total pages ({}).", page_num, part, total_pages),
                    )));
                }
                pages.insert(page_num - 1); // Convert to 0-indexed
            }
        } else {
            let page_num = part.trim().parse::<u32>().map_err(|_| {
                NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid page number: {}", part),
                ))
            })?;

            if page_num == 0 {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Page numbers must be 1 or greater.".to_string(),
                )));
            }

            if page_num > total_pages {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Page {} exceeds total pages ({}).", page_num, total_pages),
                )));
            }
            pages.insert(page_num - 1); // Convert to 0-indexed
        }
    }

    Ok(pages.into_iter().collect()) // Convert BTreeSet to Vec
}

async fn process_and_save_file(
    file_path: &str,
    client: &dyn AiProvider,
    output_dir: Option<&str>,
    pages_per_batch: u32,
    selected_pages_arg: Option<Vec<u32>>, // Renamed parameter to avoid conflict
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

    // Load progress tracker
    let mut tracker = ProgressTracker::load()?;
    
    if path.extension().and_then(|ext| ext.to_str()) == Some("pdf") {
        // Process PDF file page by page
        let (pdf, total_pages) = process_pdf(file_path)?;
        
        // Determine which pages to process based on selected_pages_arg
        let pages_to_process = if let Some(mut pages) = selected_pages_arg {
            // If specific pages are selected, filter out already processed ones if resuming
            let last_processed = tracker.get_progress(file_path)
                .map(|p| p.last_processed_page)
                .unwrap_or(0);
            
            pages.retain(|&p_idx| p_idx >= last_processed);
            
            if pages.is_empty() && tracker.get_progress(file_path).is_some() {
                 progress_bar.println(format!("{} {}", "ℹ️".cyan(), "All specified pages already processed. Skipping.".cyan()));
                 tracker.mark_completed(file_path); // Mark as completed if all selected pages are done
                 tracker.save()?;
                 return Ok(());
            }
            pages.sort_unstable(); // Ensure pages are sorted for sequential processing
            pages
        } else {
            // If no specific pages are selected, process all pages from current progress
            let progress = tracker.get_progress(file_path)
                .map(|p| p.last_processed_page)
                .unwrap_or(0);
            (progress..total_pages).collect()
        };

        if pages_to_process.is_empty() {
            progress_bar.println(format!("{} {}", "ℹ️".cyan(), "No pages to process.".cyan()));
            // If no pages to process, and it's not a full document completion, we should still ensure progress is saved if it was previously started.
            if tracker.get_progress(file_path).is_some() {
                tracker.mark_completed(file_path);
                tracker.save()?;
            }
            return Ok(());
        }

        // Read existing markdown content if it exists
        // This is necessary because we might be appending to a partially converted file.
        let mut markdown_content = if Path::new(&output_path).exists() {
            fs::read_to_string(&output_path)?
        } else {
            String::new()
        };
        
        let mut processed_pages_count_in_session = 0;
        let total_selected_pages_count = pages_to_process.len() as u32;

        // Iterate through pages in batches using the determined 'pages_to_process'
        let mut pages_iter = pages_to_process.into_iter().peekable();
        while let Some(&_current_0_indexed_page) = pages_iter.peek() {
            let mut batch_data: Vec<crate::file_utils::FileData> = Vec::new();
            let mut pages_in_current_batch: Vec<u32> = Vec::new(); // Store 0-indexed pages in this batch

            for _i in 0..pages_per_batch {
                if let Some(page_num_0_indexed) = pages_iter.next() {
                    progress_bar.println(format!(
                        "{} {}",
                        "📄".blue(),
                        format!("Processing page {} of {}", page_num_0_indexed + 1, total_pages).blue()
                    ));
                    let file_data = extract_page_as_image(&pdf, page_num_0_indexed)?;
                    batch_data.push(file_data);
                    pages_in_current_batch.push(page_num_0_indexed);
                } else {
                    break; // No more pages in selection or batch
                }
            }
            
            if batch_data.is_empty() {
                break; // Should not happen given the outer loop condition, but as a safeguard
            }

            progress_bar.set_message(format!("{}", "Sending batch to your AI model...".yellow()));

            let page_markdown = client.send_request(batch_data).await?;
            
            // Add page separator if there's existing content AND new content to add
            if !markdown_content.is_empty() && !page_markdown.is_empty() {
                markdown_content.push_str("\n\n---\n\n");
            }
            markdown_content.push_str(&page_markdown);

            // Save content after each batch
            fs::write(&output_path, &markdown_content)?;
            progress_bar.println(format!(
                "{} {}",
                "💾".green(),
                format!("Progress saved to '{}'", output_path.cyan()).green()
            ));

            // Update progress for the last page in the current batch
            if let Some(&last_page_processed_0_indexed) = pages_in_current_batch.last() {
                tracker.update_progress(
                    file_path.to_string(),
                    ProcessingProgress {
                        last_processed_page: last_page_processed_0_indexed + 1, // Store 1-indexed for clarity
                        total_pages,
                    },
                );
            }
            tracker.save()?;

            processed_pages_count_in_session += pages_in_current_batch.len() as u32;
        }

        // Final save (might be redundant but ensures file is written fully)
        fs::write(&output_path, markdown_content)?;
        
        // Mark as completed only if all *initially selected* pages were processed
        // or if it was a full document conversion and it's truly finished.
        if processed_pages_count_in_session == total_selected_pages_count {
            tracker.mark_completed(file_path);
            tracker.save()?;
        }

        progress_bar.println(format!(
            "{} {}",
            "✔".green(),
            format!("Markdown saved to '{}'", output_path.cyan()).green()
        ));
    } else {
        // Handle non-PDF files as before
        let file_data = file_utils::process_file(file_path)?;
        progress_bar.println(format!(
            "{} {}",
            "✔".green(),
            "File read successfully.".green()
        ));

        progress_bar.set_message(format!("{}", "Sending to your AI model...".yellow()));

        let markdown = client.send_request(vec![file_data]).await?;
        progress_bar.println(format!("{} {}", "✔".green(), "Received response.".green()));

        match std::fs::write(&output_path, markdown) {
            Ok(_) => {
                progress_bar.println(format!(
                    "{} {}",
                    "✔".green(),
                    format!("Markdown saved to '{}'", output_path.cyan()).green()
                ));
            }
            Err(e) => {
                progress_bar.println(format!(
                    "{} {}",
                    "✖".red(),
                    format!("Failed to save file to '{}'. Error: {}", &output_path, e).red()
                ));
                return Err(e.into());
            }
        }
    }
    
    Ok(())
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
                if let Some(config_path) = config::get_config_path() {
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
                if let Some(config_path) = config::get_config_path() {
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
            pages_per_batch,
            pages, // Capture the new 'pages' argument
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

            if pages_per_batch > 30 {
                return Err(NotedError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "pages_per_batch cannot exceed 30",
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
                        // For directory processing, pages argument is usually not applicable
                        // or would apply to each PDF within the directory.
                        // For simplicity, we'll assume it's only for single PDF processing.
                        // If it were to apply here, you'd need to re-parse it for each PDF.
                        if let Err(e) = process_and_save_file(
                            file_path_str,
                            client.as_ref(),
                            output.as_deref(),
                            pages_per_batch,
                            None, // No specific pages for batch directory processing
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

                // For single file, check if it's a PDF and if pages argument is provided
                let selected_pages = if path_str.ends_with(".pdf") {
                    if let Some(page_selection_str) = pages {
                        // We need total pages to validate selection first
                        let (pdf_dummy, total_pages) = process_pdf(path_str)?;
                        drop(pdf_dummy); // Drop PDF as we only need total_pages here
                        Some(parse_page_ranges(&page_selection_str, total_pages)?)
                    } else {
                        None
                    }
                } else {
                    None // Pages argument is only for PDF files
                };

                let progress_bar = ProgressBar::new(1); // Set to 1 as it's a single file (or handled internally for PDF pages)
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
                    pages_per_batch,
                    selected_pages, // Pass the parsed selected pages
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
    if let Err(e) = run().await {
        eprintln!("{} {}", "✖".red(), e.to_string().red());
        std::process::exit(1);
    }
}