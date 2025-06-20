mod cli;
mod file_utils;
mod gemini_client;
use std::path::Path;

use clap::Parser;
use cli::Cli;
use gemini_client::GeminiClient;

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

    println!("-> Sending request to Gemini");
    let response = match client.send_request(file_data).await {
        Ok(res) => {
            println!("Successfully received response from Gemini");
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
            "-> Could not find text in Gemini response for {:?}. Skipping.",
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
        Ok(_) => println!("Successfully saved markdown to {}", output_path),
        Err(e) => {
            eprintln!("Error saving the markdown file: {}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let input_path = Path::new(&args.file_path);
    let api_key = match args.api_key {
        Some(key) => key,
        None => {
            eprintln!("Error: Gemini API key is not set.");
            std::process::exit(1);
        }
    };

    let client = GeminiClient::new(api_key);

    if input_path.is_dir() {
        for entry in std::fs::read_dir(input_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let mime_type = file_utils::get_file_mime_type(path.to_str().unwrap());
            if path.is_file() && mime_type != "application/octet-stream" {
                process_and_save_file(path.to_str().unwrap(), &client, args.output.as_deref())
                    .await;
            }
        }
    } else {
        process_and_save_file(&args.file_path, &client, args.output.as_deref()).await;
    }
}
