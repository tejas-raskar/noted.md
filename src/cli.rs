use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A command-line tool to convert handwritten notes into clean and readable Markdown files",
    long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Convert files to Markdown format
    Convert {
        /// Path to a file or directory to convert
        #[arg(required = true)]
        path: String,

        /// Output directory to save converted files
        #[arg(
            short,
            long,
            help = "Directory where converted markdown files will be saved"
        )]
        output: Option<String>,

        /// API key for conversion
        #[arg(long, env = "GEMINI_API_KEY", hide_env_values = true)]
        api_key: Option<String>,
    },

    /// Configure notedmd settings
    Config {
        /// Set your Gemini API key
        #[arg(long, help = "Set your Gemini API key for future use")]
        set_api_key: Option<String>,

        /// Show config file location
        #[arg(long, help = "Display the path to your configuration file")]
        show_path: bool,
    },
}
