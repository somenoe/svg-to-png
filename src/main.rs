use anyhow::Result;
use clap::Parser;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

/// SVG to PNG converter with file watching
#[derive(Parser, Debug)]
#[command(author, about, long_about = None, disable_version_flag = true)]
struct Args {
    /// Input folder to watch for SVG files
    #[arg(short, long, default_value = ".")]
    input: String,

    /// Output folder for PNG files
    #[arg(short, long, default_value = "out")]
    output: String,

    /// Convert all existing SVG files on startup
    #[arg(short = 'e', long, default_value_t = false)]
    convert_existing: bool,

    /// One-time conversion without watching (no file watcher)
    #[arg(short = 'n', long, default_value_t = false)]
    no_watch: bool,

    /// Print version
    #[arg(short = 'v', long = "version")]
    version: bool,
}

/// Convert an SVG file to PNG
fn convert(svg: &Path, png: &Path) -> Result<()> {
    // Read SVG file data
    let svg_data = fs::read(svg)?;

    // Create font database and load system fonts for text rendering
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    // Parse SVG with font database for proper text rendering
    let opt = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };

    let tree = usvg::Tree::from_data(&svg_data, &opt)?;

    // Get the SVG size
    let size = tree.size();

    // Create a pixmap
    let mut pixmap = tiny_skia::Pixmap::new(size.width() as u32, size.height() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Render SVG to pixmap
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Save as PNG
    pixmap.save_png(png)?;

    println!("Converted: {} → {}", svg.display(), png.display());

    Ok(())
}

/// Convert all existing SVG files in a directory
fn convert_existing_files(input_path: &Path, output_path: &Path) -> Result<()> {
    for entry in fs::read_dir(input_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("svg")
            && let Some(stem) = path.file_stem()
        {
            let mut png_path = output_path.to_path_buf();
            png_path.push(stem);
            png_path.set_extension("png");

            if let Err(e) = convert(&path, &png_path) {
                eprintln!("Error converting {}: {}", path.display(), e);
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle version flag
    if args.version {
        println!("svg-to-png {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Create input and output directories
    let input_path = PathBuf::from(&args.input);
    let output_path = PathBuf::from(&args.output);

    fs::create_dir_all(&input_path)?;
    fs::create_dir_all(&output_path)?;

    // Convert existing files if requested or in no-watch mode
    if args.convert_existing || args.no_watch {
        println!("Converting existing SVG files...");
        convert_existing_files(&input_path, &output_path)?;
    }

    // If no-watch mode, exit after converting
    if args.no_watch {
        println!("Conversion complete. Exiting (no-watch mode).");
        return Ok(());
    }

    // Set up file watcher
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&input_path, RecursiveMode::Recursive)?;

    println!("Watching {:?}", args.input);

    // Main event loop
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(Event {
                kind: EventKind::Create(_) | EventKind::Modify(_),
                paths,
                ..
            })) => {
                for path in paths {
                    // Only process .svg files
                    if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                        // Get the file stem (filename without extension)
                        if let Some(stem) = path.file_stem() {
                            // Create output path with .png extension
                            let mut png_path = output_path.clone();
                            png_path.push(stem);
                            png_path.set_extension("png");

                            // Convert the file
                            if let Err(e) = convert(&path, &png_path) {
                                eprintln!("Error converting {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
            Ok(Ok(_)) => {
                // Ignore other event types
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Just continue watching
            }
            Err(e) => {
                eprintln!("Channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a simple valid SVG
    fn create_test_svg() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <rect x="10" y="10" width="80" height="80" fill="blue"/>
</svg>"#
            .to_string()
    }

    /// Helper function to create an SVG with text
    fn create_svg_with_text() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
    <text x="10" y="50" font-family="Arial" font-size="20" fill="black">Hello World</text>
</svg>"#
            .to_string()
    }

    #[test]
    fn test_convert_valid_svg() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        let png_path = temp_dir.path().join("test.png");

        // Create a test SVG file
        fs::write(&svg_path, create_test_svg()).unwrap();

        // Convert it
        let result = convert(&svg_path, &png_path);

        assert!(result.is_ok(), "Conversion should succeed");
        assert!(png_path.exists(), "PNG file should be created");

        // Verify the PNG file has content
        let metadata = fs::metadata(&png_path).unwrap();
        assert!(metadata.len() > 0, "PNG file should not be empty");
    }

    #[test]
    fn test_convert_svg_with_text() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("text.svg");
        let png_path = temp_dir.path().join("text.png");

        // Create an SVG with text
        fs::write(&svg_path, create_svg_with_text()).unwrap();

        // Convert it
        let result = convert(&svg_path, &png_path);

        assert!(result.is_ok(), "Conversion with text should succeed");
        assert!(png_path.exists(), "PNG file should be created");
    }

    #[test]
    fn test_convert_invalid_svg() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("invalid.svg");
        let png_path = temp_dir.path().join("invalid.png");

        // Create an invalid SVG file
        fs::write(&svg_path, "not a valid svg").unwrap();

        // Try to convert it
        let result = convert(&svg_path, &png_path);

        assert!(result.is_err(), "Conversion should fail for invalid SVG");
        // PNG should not be created on error
        assert!(
            !png_path.exists(),
            "PNG file should not be created on error"
        );
    }

    #[test]
    fn test_convert_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("nonexistent.svg");
        let png_path = temp_dir.path().join("output.png");

        // Try to convert a file that doesn't exist
        let result = convert(&svg_path, &png_path);

        assert!(
            result.is_err(),
            "Conversion should fail for nonexistent file"
        );
    }

    #[test]
    fn test_convert_existing_files_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");

        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create multiple SVG files
        fs::write(input_dir.join("test1.svg"), create_test_svg()).unwrap();
        fs::write(input_dir.join("test2.svg"), create_test_svg()).unwrap();
        fs::write(input_dir.join("test3.svg"), create_svg_with_text()).unwrap();

        // Convert all existing files
        let result = convert_existing_files(&input_dir, &output_dir);

        assert!(result.is_ok(), "Batch conversion should succeed");

        // Verify all PNG files were created
        assert!(
            output_dir.join("test1.png").exists(),
            "test1.png should exist"
        );
        assert!(
            output_dir.join("test2.png").exists(),
            "test2.png should exist"
        );
        assert!(
            output_dir.join("test3.png").exists(),
            "test3.png should exist"
        );
    }

    #[test]
    fn test_convert_existing_files_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("empty_input");
        let output_dir = temp_dir.path().join("empty_output");

        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Convert with empty directory
        let result = convert_existing_files(&input_dir, &output_dir);

        assert!(result.is_ok(), "Empty directory conversion should succeed");
    }

    #[test]
    fn test_convert_existing_files_mixed_types() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("mixed");
        let output_dir = temp_dir.path().join("output_mixed");

        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create mixed file types
        fs::write(input_dir.join("test.svg"), create_test_svg()).unwrap();
        fs::write(input_dir.join("readme.txt"), "Some text").unwrap();
        fs::write(input_dir.join("data.json"), "{}").unwrap();

        // Convert - should only process .svg files
        let result = convert_existing_files(&input_dir, &output_dir);

        assert!(result.is_ok(), "Mixed directory conversion should succeed");

        // Only the SVG should be converted
        assert!(
            output_dir.join("test.png").exists(),
            "test.png should exist"
        );
        assert!(
            !output_dir.join("readme.png").exists(),
            "readme.png should not exist"
        );
        assert!(
            !output_dir.join("data.png").exists(),
            "data.png should not exist"
        );
    }

    #[test]
    fn test_convert_with_subdirectory_output() {
        let temp_dir = TempDir::new().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        let output_subdir = temp_dir.path().join("output").join("subdir");
        fs::create_dir_all(&output_subdir).unwrap();
        let png_path = output_subdir.join("test.png");

        fs::write(&svg_path, create_test_svg()).unwrap();

        let result = convert(&svg_path, &png_path);

        assert!(result.is_ok(), "Conversion to subdirectory should succeed");
        assert!(png_path.exists(), "PNG in subdirectory should exist");
    }
}
