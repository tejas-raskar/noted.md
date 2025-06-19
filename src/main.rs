mod cli;
mod file_utils;
mod gemini_client;
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

    match client.send_request(encoded_data).await {
        Ok(response) => println!("{:#?}", response),
        Err(e) => eprintln!("{}", e),
    }

    if let Some(output_path) = args.output {
        println!("Output will be saved to: {}", output_path);
    }
}
