mod cli;
mod file_utils;
mod gemini_client;
use std::path::Path;

use clap::Parser;
use cli::Cli;
use gemini_client::GeminiClient;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let encoded_data = match file_utils::process_image(&args.file_path) {
        Ok(encoded_image) => {
            println!("Image processed successfully.");
            encoded_image
        }
        Err(e) => {
            eprintln!("Error processing image: {}", e);
            std::process::exit(1);
        }
    };

    let api_key = match args.api_key {
        Some(key) => key,
        None => {
            eprintln!("Error: Gemini API key is not set.");
            std::process::exit(1);
        }
    };

    let client = GeminiClient::new(api_key);
    println!("Sending request to Gemini...");

    let response = match client.send_request(encoded_data).await {
        Ok(res) => {
            println!("Successfully received response from Gemini");
            res
        }
        Err(e) => {
            eprintln!("Error calling Gemini: {}", e);
            std::process::exit(1);
        }
    };

    let markdown_text = response
        .candidates
        .get(0)
        .and_then(|candidate| candidate.content.parts.get(0))
        .map(|part| &part.text)
        .unwrap_or_else(|| {
            eprintln!("Error parsing the response");
            std::process::exit(1);
        });
    let x: &[_] = &['`', 'm', 'a', 'r', 'k', 'd', 'o', 'w', 'n'];
    let cleaned_markdown = markdown_text.trim_start_matches(x).trim_end_matches(x);

    let output_path = args
        .output
        .map(|path_str| {
            let path = Path::new(&path_str);

            if path.is_dir() {
                let file_name = Path::new(&args.file_path).file_name().unwrap_or_else(|| {
                    eprintln!("Error determining the file name.");
                    std::process::exit(1);
                });
                let final_path = path.join(file_name);
                final_path
                    .with_extension("md")
                    .to_string_lossy()
                    .into_owned()
            } else {
                if path_str.ends_with('/') || path_str.ends_with('\\') {
                    let file_name = Path::new(&args.file_path).file_name().unwrap_or_else(|| {
                        eprintln!("Error determining the file name.");
                        std::process::exit(1);
                    });
                    path.join(file_name)
                        .with_extension("md")
                        .to_string_lossy()
                        .into_owned()
                } else {
                    path_str
                }
            }
        })
        .unwrap_or_else(|| {
            let input_path = Path::new(&args.file_path);
            input_path
                .with_extension("md")
                .to_string_lossy()
                .into_owned()
        });

    match std::fs::write(&output_path, cleaned_markdown) {
        Ok(_) => println!("Successfully saved markdown to {}", output_path),
        Err(e) => {
            eprintln!("Error saving the markdown file: {}", e);
            std::process::exit(1);
        }
    }
}
