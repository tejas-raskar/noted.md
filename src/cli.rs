use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Convert {
        #[arg(required = true)]
        path: String,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(long, env = "GEMINI_API_KEY")]
        api_key: Option<String>,
    },

    Config {
        #[arg(long)]
        set_api_key: Option<String>,

        #[arg(long)]
        show_path: bool,
    },
}
