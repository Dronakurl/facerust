use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Create models directory if it doesn't exist
    let models_dir = Path::new("models");
    if !models_dir.exists() {
        fs::create_dir_all(models_dir).expect("Failed to create models directory");
    }

    // Model URLs and file names
    let models = [
        (
            "https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx",
            "models/face_detection_yunet_2023mar.onnx"
        ),
        (
            "https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx", 
            "models/face_recognition_sface_2021dec.onnx"
        ),
    ];

    for (url, filename) in &models {
        download_if_missing(url, filename);
    }
}

fn download_if_missing(url: &str, filename: &str) {
    let path = Path::new(filename);

    if path.exists() {
        println!("cargo:warning=Model already exists: {filename}");
        return;
    }

    println!("cargo:warning=Downloading model: {url} -> {filename}");

    // Try to download the file
    match download_file(url, filename) {
        Ok(_) => {
            println!("cargo:warning=✓ Successfully downloaded: {filename}");
        }
        Err(e) => {
            eprintln!("cargo:warning=⚠ Failed to download {filename}: {e}");
            eprintln!("cargo:warning=Please download manually from: {url}");
        }
    }
}

fn download_file(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Use curl if available (most systems have it)
    if which("curl") {
        let output = std::process::Command::new("curl")
            .arg("-L") // Follow redirects
            .arg("-f") // Fail on HTTP errors
            .arg("-s") // Silent
            .arg("-o")
            .arg(filename)
            .arg(url)
            .output()?;

        if !output.status.success() {
            return Err(format!("curl failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        return Ok(());
    }

    Err("curl found. Please install or download the models manually.".into())
}

fn which(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
