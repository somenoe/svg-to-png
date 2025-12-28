use anyhow::Result;
use clap::Parser;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
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
    let mut opt = usvg::Options::default();
    opt.fontdb = fontdb;
    opt.keep_named_groups = false;

    let tree = usvg::Tree::from_data(&svg_data, &opt.to_ref())?;

    // Get the SVG size
    let size = tree.svg_node().size;

    // Create a pixmap
    let mut pixmap = tiny_skia::Pixmap::new(size.width() as u32, size.height() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Render SVG to pixmap
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to render SVG"))?;

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

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("svg") {
            if let Some(stem) = path.file_stem() {
                let mut png_path = output_path.to_path_buf();
                png_path.push(stem);
                png_path.set_extension("png");

                if let Err(e) = convert(&path, &png_path) {
                    eprintln!("Error converting {}: {}", path.display(), e);
                }
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
