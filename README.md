<div align="center">
  <pre>
          ███╗   ██╗ ██████╗ ████████╗███████╗██████╗    ███╗   ███╗██████╗
          ████╗  ██║██╔═══██╗╚══██╔══╝██╔════╝██╔══██╗   ████╗ ████║██╔══██╗
          ██╔██╗ ██║██║   ██║   ██║   █████╗  ██║  ██║   ██╔████╔██║██║  ██║
          ██║╚██╗██║██║   ██║   ██║   ██╔══╝  ██║  ██║   ██║╚██╔╝██║██║  ██║
          ██║ ╚████║╚██████╔╝   ██║   ███████╗██████╔╝██╗██║ ╚═╝ ██║██████╔╝
          ╚═╝  ╚═══╝ ╚═════╝    ╚═╝   ╚══════╝╚═════╝ ╚═╝╚═╝     ╚═╝╚═════╝
  </pre>
</div>

<p align="center">
  <strong>A command-line tool to convert handwritten notes into a clean and readable Markdown file.</strong>
</p>

<p align="center">
  <a href="https://github.com/tejas-raskar/noted.md/actions"><img src="https://github.com/tejas-raskar/noted.md/actions/workflows/release.yml/badge.svg" alt="Build Status"></a>
  <a href="http://github.com/tejas-raskar/noted.md/releases"><img src="https://img.shields.io/github/v/tag/tejas-raskar/noted.md" alt="Version"></a>
  <a href="http://github.com/tejas-raskar/noted.md/releases"><img src="https://img.shields.io/github/downloads/tejas-raskar/noted.md/total?color=red" alt="Downloads"></a>
  <a href="https://github.com/tejas-raskar/noted.md/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

---

`noted.md` is a CLI tool that uses LLMs to convert your handwritten text into markdown files. It's an interactive program that accepts pdfs, jpg, jpeg, png as an input and processes them accordingly. It can recognize mathematical equations too and can correctly format them in LaTeX. And if you have bunch of files to convert them at once, `noted.md` supports batch processing too!


https://github.com/user-attachments/assets/5e2f4ab5-2043-4ea4-b95d-bf63e36ce9d9


## Installation

`noted.md` can be installed on macOS, Linux, and Windows.

### macOS & Linux (Recommended: Homebrew)

For the easiest installation on macOS and Linux, use Homebrew:

```bash
brew tap tejas-raskar/noted.md
brew install notedmd
```

To update `noted.md` to the latest version:

```bash
brew upgrade notedmd
```

### Manual Download (Windows)

For Windows, download the latest `.zip` archive from the [Releases page](https://github.com/tejas-raskar/noted.md/releases/latest). Extract the contents and add the `bin` directory to your system's PATH.

### Building from Source

If you prefer to build from source, clone the repository and use Cargo:

```bash
git clone https://github.com/tejas-raskar/noted.md.git
cd noted.md
cargo build --release
# The executable will be in target/release/notedmd
```

## Usage

The typical workflow is:
1.  **Configure your AI provider**: Use `notedmd config --edit` for a guided setup.
2.  **Convert your files**: Use `notedmd convert <path>` to process your notes.

### Commands

| Command           | Description                                                                          |
| ----------------- | ------------------------------------------------------------------------------------ |
| `notedmd convert` | Converts a file or all supported files in a directory into Markdown.                 |
| `notedmd config`  | Manages the AI provider configuration. Shows the current config if no flags are used. |

---

## Configuration

### Interactive Setup (Recommended)

For first-time users, the interactive setup is the easiest way to get started. Run:
```bash
notedmd config --edit
```
This will guide you through selecting an AI provider (Gemini, Claude, or Ollama) and entering the necessary credentials, such as API keys or server details.

### AI Providers

You can choose between three AI providers.

#### Gemini and Claude APIs
You will need an API key from your chosen provider:
- **Gemini API:** [Google AI Studio](https://aistudio.google.com/app/apikey)
- **Claude API:** [Anthropic's website](https://console.anthropic.com/dashboard)

#### Ollama
Make sure Ollama is installed and running on your local machine. You can download it from [Ollama's website](https://ollama.com/).

#### OpenAI API compatible clients
Supports all clients that are compatible with the OpenAI API. [LM Studio](https://lmstudio.ai/) for example.

### Managing Configuration via Flags

You can also manage your configuration directly using flags.

| Flag                             | Description                                                                 |
| -------------------------------- | --------------------------------------------------------------------------- |
| `--set-provider <provider>`      | Set the active provider (`gemini`, `claude`, `ollama`).                     |
| `--set-api-key <key>`            | Set the API key for Gemini.                                                 |
| `--set-claude-api-key <key>`     | Set the API key for Claude.                                                 |
| `--show`                         | Display the current configuration.                                          |
| `--show-path`                    | Show the path to your configuration file.                                   |
| `--edit`                         | Start the interactive configuration wizard.                                 |

**Examples:**
- Set the active provider to Claude:
  ```bash
  notedmd config --set-provider claude
  ```
- Set your Gemini API key:
  ```bash
  notedmd config --set-api-key YOUR_GEMINI_API_KEY
  ```

---

## Converting Files

Once configured, you can convert your handwritten notes.

| Flag                             | Description                                                                 |
| -------------------------------- | --------------------------------------------------------------------------- |
| `-o`, `--output <dir>`           | Specify a directory to save the converted Markdown file(s).                 |
| `-p`, `--prompt <prompt>`        | Add a custom prompt to override the default instructions for the LLM.       |
| `--api-key <key>`                | Temporarily override the stored API key for a single `convert` command.     |

**Examples:**

-   **Convert a single file**:
    The converted file will be saved in the same directory with a `.md` extension (e.g., `my_document.md`).
    ```bash
    notedmd convert my_document.pdf
    ```

-   **Convert a file with a custom prompt**:
    ```bash
    notedmd convert my_notes.png --prompt "Transcribe this into a bulleted list."
    ```

-   **Convert a file and save it to a different directory**:
    ```bash
    notedmd convert my_document.pdf --output ./markdown_notes/
    ```

-   **Convert all supported files in a directory**:
    ```bash
    notedmd convert ./my_project_files/
    ```

-   **Convert all files in a directory to a specific output directory**:
    ```bash
    notedmd convert ./my_project_files/ --output ./markdown_notes/
    ```

## Contributing

Contributions are welcome! If you have a feature request, bug report, or want to contribute to the code, please feel free to open an issue or a pull request on our [GitHub repository](https://github.com/tejas-raskar/noted.md).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
