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
  <a href="https://github.com/tejas-raskar/noted.md/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

---

`noted.md` is a CLI tool that uses LLMs to convert your handwritten text into markdown files. It's an interactive program that accepts pdfs, jpg, jpeg, png as an input and processes them accordingly. It can recognize mathematical equations too and can correctly format them in LaTeX. And if you have bunch of files to convert them at once, `noted.md` supports batch processing too!

![notedmd](https://github.com/user-attachments/assets/c844305f-3311-47c6-8358-4b709f81ab37)

## Installation

`noted.md` can be installed on all major operating systems.

| Binary Name | Platform & Architecture |
|------------|------------------------|
| `notedmd-v0.1.1-x86_64-unknown-linux-musl.tar.gz` | Linux (64-bit Intel/AMD) |
| `notedmd-v0.1.1-x86_64-apple-darwin.tar.gz` | macOS (64-bit Intel) |
| `notedmd-v0.1.1-aarch64-apple-darwin.tar.gz` | macOS (Apple Silicon/M series) |
| `notedmd-v0.1.1-x86_64-pc-windows-msvc.zip` | Windows (64-bit Intel/AMD) |


Choose the instructions below that match your system:
### macOS

1. Download the latest macOS archive (e.g., `notedmd-v0.1.1-aarch64-apple-darwin.tar.gz`) from the [Releases page](https://github.com/tejas-raskar/noted.md/releases/latest)
2. Open Terminal and follow the below steps:
   ```bash
   cd ~/Downloads
   tar -xzf notedmd-v0.1.1-aarch64-apple-darwin.tar.gz
   sudo mv notedmd-*/bin/notedmd /usr/local/bin/
   ```
3. Verify the installation:
   ```bash
   notedmd --version
   ```

### Windows

1. Download the latest Windows archive (e.g., `notedmd-v0.1.1-x86_64-pc-windows-msvc.zip`) from the [Releases page](https://github.com/tejas-raskar/noted.md/releases/latest)
2. Right-click the downloaded ZIP file and select "Extract All"
3. Choose a destination folder (e.g., `C:\Program Files\noted.md`)
4. Copy the extracted files to the chosen location:
   - The directory structure should look like:
     ```
     C:\Program Files\noted.md\
     ├── bin\
     │   └── notedmd.exe
     ├── LICENSE
     ├── README.md
     └── CHANGELOG.md
     ```
5. Add the bin directory to your PATH:
   - Open Start and search for "Environment Variables"
   - Click "Edit the system environment variables"
   - Click "Environment Variables"
   - Under "System Variables", find and select "Path"
   - Click "Edit" and then "New"
   - Add the path to the bin directory (e.g., `C:\Program Files\noted.md\bin`)
   - Click "OK" on all windows
6. Open a new Command Prompt and verify the installation:
   ```cmd
   notedmd --version
   ```

### Linux

1. Download the latest Linux archive (e.g., `notedmd-v0.1.1-x86_64-unknown-linux-musl.tar.gz`) from the [Releases page](https://github.com/tejas-raskar/noted.md/releases/latest)
2. Open Terminal and follow the below steps:
   ```bash
   cd ~/Downloads
   tar -xzf notedmd-v0.1.1-x86_64-unknown-linux-musl.tar.gz
   sudo mv notedmd-*/bin/notedmd /usr/local/bin/
   ```
3. Verify the installation:
   ```bash
   notedmd --version
   ```


### Building from source

If you have Rust installed, you can also build `noted.md` from source:

```bash
cargo install --git https://github.com/tejas-raskar/noted.md.git
```


## Usage

### 1. Configuration

You will need a Gemini API key before starting to convert files. You can get one from [Google AI Studio](https://aistudio.google.com/app/apikey).

Run `notedmd config` initially to configure your AI provider. Ollama support soon!

You can change the Gemini API key in one of two ways:

-   **Using the `config` command**:

    ```bash
    notedmd config --set-api-key YOUR_GEMINI_API_KEY
    ```

-   **Using the `--api-key` flag**:

    Pass the key directly with the `convert` command.

    ```bash
    notedmd convert my_file.pdf --api-key YOUR_GEMINI_API_KEY
    ```

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
