# SVG Watcher

A Rust CLI tool that watches a folder for `.svg` file changes and automatically converts them to `.png` files using the `resvg` rendering engine.

## Features

- **File Watching**: Monitors a specified directory for new or modified SVG files.
- **Automatic Conversion**: Instantly converts detected SVGs to PNG format.
- **Batch Processing**: Optional flag to convert all existing SVGs on startup.
- **Cross-Platform**: Works on Windows, macOS, and Linux.

## Installation

Ensure you have Rust installed. Then, clone the repository and build the project:

```bash
git clone <repository-url>
cd svgwatcher
cargo build --release
```

## Usage

Run the tool using `cargo run` or the built binary.

```bash
# Basic usage (watches current directory, outputs to ./out)
cargo run

# Specify input and output directories
cargo run -- --input ./svgs --output ./pngs

# Convert existing files on startup
cargo run -- --input ./svgs --output ./pngs --convert-existing
```

### Command Line Arguments

- `-i, --input <DIR>`: Input folder to watch for SVG files (default: `.`)
- `-o, --output <DIR>`: Output folder for PNG files (default: `out`)
- `-e, --convert-existing`: Convert all existing SVG files on startup (default: `false`)

## Dependencies

- [clap](https://crates.io/crates/clap) - Command-line argument parsing
- [notify](https://crates.io/crates/notify) - File system notification
- [resvg](https://crates.io/crates/resvg) - SVG rendering
- [usvg](https://crates.io/crates/usvg) - SVG parsing
- [tiny-skia](https://crates.io/crates/tiny-skia) - 2D graphics library

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
