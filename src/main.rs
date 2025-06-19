mod cli;
mod file_utils;
use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();

    match file_utils::process_image(&args.file_path) {
        Ok(encoded_data) => {
            println!(
                "Image encoded successfully. Starts with: {}...",
                &encoded_data[..30]
            )
        }
        Err(e) => eprintln!("Error processing image: {}", e),
    }

    if let Some(output_path) = args.output {
        println!("Output will be saved to: {}", output_path);
    }

    if let Some(_api_key) = args.api_key {
        println!("API key provided: ***");
    } else {
        println!("API key not provied");
    }
}
