use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

/// Helper function to get the path to the compiled binary
fn get_binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove deps directory
    path.push("svg-to-png");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Helper to create a simple test SVG
fn create_test_svg_content() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <rect x="10" y="10" width="80" height="80" fill="red"/>
</svg>"#
}

#[test]
fn test_version_flag() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("svg-to-png"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_short_version_flag() {
    let output = Command::new(get_binary_path())
        .arg("-v")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("svg-to-png"));
}

#[test]
fn test_no_watch_mode_with_existing_svgs() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("input");
    let output_dir = temp.child("output");

    input_dir.create_dir_all().unwrap();

    // Create test SVG files
    input_dir
        .child("test1.svg")
        .write_str(create_test_svg_content())
        .unwrap();
    input_dir
        .child("test2.svg")
        .write_str(create_test_svg_content())
        .unwrap();

    // Run conversion in no-watch mode
    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");

    // Verify PNG files were created
    output_dir
        .child("test1.png")
        .assert(predicate::path::exists());
    output_dir
        .child("test2.png")
        .assert(predicate::path::exists());
}

#[test]
fn test_convert_existing_flag() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("svgs");
    let output_dir = temp.child("pngs");

    input_dir.create_dir_all().unwrap();

    // Create test SVG
    input_dir
        .child("circle.svg")
        .write_str(create_test_svg_content())
        .unwrap();

    // Run with convert-existing and no-watch
    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--convert-existing")
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    output_dir
        .child("circle.png")
        .assert(predicate::path::exists());
}

#[test]
fn test_creates_output_directory() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("input");
    let output_dir = temp.child("deeply").child("nested").child("output");

    input_dir.create_dir_all().unwrap();
    input_dir
        .child("test.svg")
        .write_str(create_test_svg_content())
        .unwrap();

    // Output directory doesn't exist yet
    assert!(!output_dir.path().exists());

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify output directory was created
    assert!(output_dir.path().exists());
    output_dir
        .child("test.png")
        .assert(predicate::path::exists());
}

#[test]
fn test_empty_directory_no_crash() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("empty");
    let output_dir = temp.child("out");

    input_dir.create_dir_all().unwrap();

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Should handle empty directory gracefully"
    );
}

#[test]
fn test_mixed_file_types() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("mixed");
    let output_dir = temp.child("output");

    input_dir.create_dir_all().unwrap();

    // Create various file types
    input_dir
        .child("image.svg")
        .write_str(create_test_svg_content())
        .unwrap();
    input_dir
        .child("readme.txt")
        .write_str("Some text")
        .unwrap();
    input_dir.child("data.json").write_str("{}").unwrap();
    input_dir
        .child("script.py")
        .write_str("print('hello')")
        .unwrap();

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Only SVG should be converted
    output_dir
        .child("image.png")
        .assert(predicate::path::exists());
    output_dir
        .child("readme.png")
        .assert(predicate::path::missing());
    output_dir
        .child("data.png")
        .assert(predicate::path::missing());
    output_dir
        .child("script.png")
        .assert(predicate::path::missing());
}

#[test]
fn test_preserves_filename() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.child("input");
    let output_dir = temp.child("output");

    input_dir.create_dir_all().unwrap();

    // Create SVG with specific name
    input_dir
        .child("my-special-file.svg")
        .write_str(create_test_svg_content())
        .unwrap();

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.path())
        .arg("--output")
        .arg(output_dir.path())
        .arg("--no-watch")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify filename is preserved (only extension changes)
    output_dir
        .child("my-special-file.png")
        .assert(predicate::path::exists());
}
