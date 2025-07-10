use crate::Config;
use colored::Colorize;

pub fn ascii_art() {
    println!(
        "{}",
        r"

          ███╗   ██╗ ██████╗ ████████╗███████╗██████╗    ███╗   ███╗██████╗
          ████╗  ██║██╔═══██╗╚══██╔══╝██╔════╝██╔══██╗   ████╗ ████║██╔══██╗
          ██╔██╗ ██║██║   ██║   ██║   █████╗  ██║  ██║   ██╔████╔██║██║  ██║
          ██║╚██╗██║██║   ██║   ██║   ██╔══╝  ██║  ██║   ██║╚██╔╝██║██║  ██║
          ██║ ╚████║╚██████╔╝   ██║   ███████╗██████╔╝██╗██║ ╚═╝ ██║██████╔╝
          ╚═╝  ╚═══╝ ╚═════╝    ╚═╝   ╚══════╝╚═════╝ ╚═╝╚═╝     ╚═╝╚═════╝
        "
        .bright_blue()
    );
    println!(
        "{}",
        "-------------------------------------------------".dimmed()
    );
}

pub fn print_clean_config(config: Config) {
    println!("{}", "noted.md Configuration".bold());
    println!("-------------------------");

    if let Some(provider) = config.active_provider {
        println!("Active Provider: {}", provider.green());
    } else {
        println!("Active Provider: {}", "Not Set".yellow());
    }

    println!("{}", "Gemini".bold());
    if let Some(gemini_config) = config.gemini {
        let api_key = format!(
            "{:.3}***************** (hidden for security)",
            gemini_config.api_key
        );
        println!("  API Key: {}", api_key);
    } else {
        println!("  (Not Configured)");
    }

    println!("{}", "Claude".bold());
    if let Some(claude_config) = config.claude {
        let api_key = format!(
            "{:.3}***************** (hidden for security)",
            claude_config.api_key
        );
        println!("  API Key: {}", api_key);
        println!("  Model:   {}", claude_config.model);
    } else {
        println!("  (Not Configured)");
    }

    println!("{}", "Ollama".bold());
    if let Some(ollama_config) = config.ollama {
        println!("  URL:     {}", ollama_config.url);
        println!("  Model:   {}", ollama_config.model);
    } else {
        println!("  (Not Configured)");
    }

    println!("{}", "OpenAI (Compatible)".bold());
    if let Some(openai_config) = config.openai {
        println!("  URL:     {}", openai_config.url);
        println!("  Model:   {}", openai_config.model);
        let api_key = if openai_config.api_key.is_none() {
            "API key empty.".to_string()
        } else {
            openai_config.api_key.unwrap()
        };

        println!("  API Key: {}", api_key);
    } else {
        println!("  (Not Configured)");
    }
}
