use clap::{Arg, Command};
use facerust::FaceRecognition;
use opencv::{
    imgcodecs::{imread, imwrite, IMREAD_COLOR},
    prelude::*,
};
use std::path::Path;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let matches = Command::new("Face Recognition CLI Tool")
        .version("1.0")
        .author("Your Name")
        .about("Rust implementation of face recognition CLI")
        .arg(
            Arg::new("image")
                .short('i')
                .long("image")
                .value_name("FILE")
                .help("Path to the input image")
                .default_value("/app/media/testdata/IMG.jpg"),
        )
        .arg(
            Arg::new("db")
                .short('d')
                .long("db")
                .value_name("DIR")
                .help("Path to the faces database")
                .default_value("/app/media/db"),
        )
        .arg(
            Arg::new("test-mode")
                .short('t')
                .long("test-mode")
                .help("Run in mode to test database update")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let image_path = matches.get_one::<String>("image").unwrap();
    let db_path = matches.get_one::<String>("db").unwrap();
    let test_mode = matches.get_flag("test-mode");

    // Check if files exist
    if !Path::new(image_path).exists() {
        eprintln!("Error: Image file does not exist: {}", image_path);
        std::process::exit(1);
    }

    if !Path::new(db_path).exists() {
        eprintln!("Error: Database directory does not exist: {}", db_path);
        std::process::exit(1);
    }

    if test_mode {
        test_mode_run(image_path, db_path).await?;
    } else {
        simple_run(image_path, db_path).await?;
    }

    Ok(())
}

/// Simple face recognition run on one image
async fn simple_run(image_path: &str, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running simple face recognition...");

    // Load image
    let mut frame = imread(image_path, IMREAD_COLOR)?;
    if frame.empty() {
        return Err(format!("Could not load image: {}", image_path).into());
    }

    // Initialize face recognition
    let mut face_recognition = FaceRecognition::new(
        Some("models/face_detection_yunet_2023mar.onnx"),
        Some("models/face_recognition_sface_2021dec.onnx"),
        Some(1000),
    )?;

    // Load database
    face_recognition
        .load_persons_db(db_path, false, false)
        .await?;

    // Run face recognition
    let results = face_recognition.run(&mut frame, 0.4, true).await?;

    for (i, result) in results.iter().enumerate() {
        info!("Face {}: {}", i + 1, result.to_string());
    }

    // Save result
    let output_path = "./media/result.jpg";
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    imwrite(output_path, &frame, &opencv::core::Vector::new())?;
    info!("Result saved to: {}", output_path);

    Ok(())
}

/// Test mode to verify database update mechanism
async fn test_mode_run(image_path: &str, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Face Recognition Async Database Test ===");

    // Initialize face recognition
    info!("1. Initializing FaceRecognition...");
    let mut face_recognition = FaceRecognition::new(
        Some("models/face_detection_yunet_2023mar.onnx"),
        Some("models/face_recognition_sface_2021dec.onnx"),
        Some(1000),
    )?;

    // Load the initial database
    info!("2. Loading initial persons database from: {}", db_path);
    face_recognition
        .load_persons_db(db_path, false, false)
        .await?;

    // Start watching for database changes (check every 2 seconds for faster testing)
    info!("3. Starting database watcher (check interval: 2 seconds)...");
    face_recognition.start_watching(2).await?;

    // Load and process the test image
    info!("4. Loading test image: {}", image_path);
    let frame = imread(image_path, IMREAD_COLOR)?;
    if frame.empty() {
        return Err(format!("Could not load image: {}", image_path).into());
    }

    info!("5. Running face recognition on test image...");
    let result = face_recognition
        .run_one_face(frame.clone(), 0.4, false)
        .await?;
    info!("Found name: {}", result.to_string());

    // Wait a bit to let any initial processing complete
    info!("6. Waiting 3 seconds...");
    sleep(Duration::from_secs(3)).await;

    // Now trigger a database change by creating an empty JPG file
    info!("7. Triggering database change by creating an empty JPG file...");
    let subfolder_path = Path::new(db_path).join("misterx");
    let test_file_path = subfolder_path.join("testme.jpg");

    // Ensure the subfolder exists
    if !subfolder_path.exists() {
        std::fs::create_dir_all(&subfolder_path)?;
    }

    // Create an empty JPG file (white image)
    let white_image = opencv::core::Mat::new_rows_cols_with_default(
        400,
        400,
        opencv::core::CV_8UC3,
        opencv::core::Scalar::all(255.0),
    )?;

    imwrite(
        test_file_path.to_str().unwrap(),
        &white_image,
        &opencv::core::Vector::new(),
    )?;

    if test_file_path.exists() {
        info!("   Created test file: {}", test_file_path.display());
    } else {
        warn!("   Error: Could not create test file");
    }

    // Wait for the watcher to detect the change and reload
    info!("8. Waiting 10 seconds for database watcher to detect change...");
    info!("   (Watch the debug output for 'Database folder changed, reloading...')");
    sleep(Duration::from_secs(10)).await;

    // Run face recognition again to show it's still working
    info!("9. Running face recognition again after database reload...");
    let result = face_recognition.run_one_face(frame, 0.4, false).await?;
    info!("Found name: {}", result.name);

    // Clean up the test file
    info!("10. Cleaning up test file...");
    if test_file_path.exists() {
        std::fs::remove_file(&test_file_path)?;
        info!("    Test file removed.");
    }

    // Stop watching
    info!("11. Stopping database watcher...");
    face_recognition.stop_watching().await;

    info!("=== Test completed ===");
    info!("Expected behavior:");
    info!("- Initial database load");
    info!("- First face recognition run");
    info!("- Database change detection and automatic reload");
    info!("- Second face recognition run working normally");

    Ok(())
}
