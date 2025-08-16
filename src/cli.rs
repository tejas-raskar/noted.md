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

        /// Prompt the LLM
        #[arg(short, long, help = "Add a custom prompt to pass to the LLM")]
        prompt: Option<String>,

        /// Notion Support
        #[arg(short, long, help = "Use Notion to store the generated output")]
        notion: bool,

        /// Enable few-shot learning with examples
        #[arg(long, help = "Use existing examples for few-shot learning")]
        use_examples: bool,

        /// Filter examples by tags (comma-separated, all must match)
        #[arg(long, help = "Comma-separated tags to filter examples (e.g., 'danish,journal')")]
        examples_tags: Option<String>,

        /// Limit number of examples to use
        #[arg(long, help = "Maximum number of examples to use", default_value = "3")]
        examples_count: usize,
    },

    /// Configure notedmd settings
    Config {
        /// Set your Gemini API key
        #[arg(long, help = "Set your Gemini API key for future use")]
        set_api_key: Option<String>,

        /// Set your Claude API key
        #[arg(long, help = "Set your Claude API key for future use")]
        set_claude_api_key: Option<String>,

        /// Set active provider
        #[arg(long, help = "Set the active provider")]
        set_provider: Option<String>,

        /// Show config file location
        #[arg(long, help = "Shows the location of your configuration file")]
        show_path: bool,

        /// Show config file
        #[arg(long, help = "Shows the content of your configuration")]
        show: bool,

        /// Trigger onboarding flow
        #[arg(long, help = "Edit the configuration file")]
        edit: bool,
    },

    /// Manage examples for few-shot learning
    Examples {
        #[command(subcommand)]
        action: ExampleAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ExampleAction {
    /// Add a new example
    Add {
        /// Path to the image file
        image: String,
        /// Path to the correct markdown transcription
        markdown: String,
        /// Tags to categorize this example
        #[arg(short, long)]
        tag: Vec<String>,
    },
    /// List existing examples
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Remove an example
    Remove {
        /// Example ID to remove
        id: String,
    },
    /// Show details of an example
    Show {
        /// Example ID to show
        id: String,
    },
}
