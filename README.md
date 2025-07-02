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

![notedmd](https://github.com/user-attachments/assets/c844305f-3311-47c6-8358-4b709f81ab37)

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

### 1. Configuration

Run `notedmd config` to configure your AI provider. You can choose between:
  - Gemini API **(recommended)**
  - Claude API
  - Ollama

#### Gemini and Claude APIs
You can get the API keys from their respective websites:
- **Gemini API:** [Google AI Studio](https://aistudio.google.com/app/apikey)
- **Claude API:** [Anthropic's website](https://console.anthropic.com/dashboard)

During the initial `notedmd config` setup, you will be prompted for your key.

If you wish to change the API key later, you can do so in two ways:

-   **Using the `config` command**:
    - For Gemini:
      ```bash
      notedmd config --set-api-key YOUR_GEMINI_API_KEY
      ```
    - For Claude:
      ```bash
      notedmd config --set-claude-api-key YOUR_CLAUDE_API_KEY
      ```

-   **Using the `--api-key` flag**:
    This flag overrides the active provider's API key for a single `convert` command.
    ```bash
    notedmd convert my_file.pdf --api-key YOUR_API_KEY
    ```


#### Ollama
Make sure Ollama is installed and running. You can download Ollama from [Ollama's website](https://ollama.com/).


You can see where the configuration is stored by running:
```bash
notedmd config --show-path
```

### 2. Converting Files

-   **Convert a single file**:

    The converted file will be saved in the same directory with a `.md` extension (e.g., `my_document.md`).

    ```bash
    notedmd convert my_document.pdf
    ```

-   **Convert a single file to a specific output directory**:

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
