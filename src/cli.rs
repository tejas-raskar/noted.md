use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(required = true)]
    pub file_path: String,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long, env = "GEMINI_API_KEY")]
    pub api_key: Option<String>,
}
