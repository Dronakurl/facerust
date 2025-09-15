use crate::types::{DbLoadStatus, DetectedFace, MatchResult, MatchResults};
use crate::watcher::{get_latest_mod_time, FolderWatcher};
use crate::{FaceRecognitionError, Result};
use opencv::{
    core::{Mat, Point, Ptr, Rect2i, Scalar, Size},
    imgcodecs::{imread, imwrite, IMREAD_COLOR},
    imgproc::{get_text_size, put_text, rectangle, FONT_HERSHEY_SIMPLEX, LINE_8},
    objdetect::{FaceDetectorYN, FaceRecognizerSF},
    prelude::*,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct FaceRecognition {
    detector: Ptr<FaceDetectorYN>,
    face_recognizer: Ptr<FaceRecognizerSF>,
    max_size: i32,
    db_load_status: Arc<RwLock<DbLoadStatus>>,
    features_map: Arc<RwLock<HashMap<String, Vec<Mat>>>>,
    db_path: Arc<RwLock<Option<PathBuf>>>,
    last_mod_time: Arc<RwLock<SystemTime>>,
    watcher: Arc<Mutex<Option<FolderWatcher>>>,
    watcher_running: Arc<AtomicBool>,
}

const SCORE_THRESHOLD: f32 = 0.5; // Lowered from 0.7 for better face detection
const NMS_THRESHOLD: f32 = 0.3;
const TOP_K: i32 = 5000;

impl FaceRecognition {
    pub fn new(
        fd_model_path: Option<&str>,
        fr_model_path: Option<&str>,
        max_size: Option<i32>,
    ) -> Result<Self> {
        let fd_path = fd_model_path.unwrap_or("./models/face_detection_yunet_2023mar.onnx");
        let fr_path = fr_model_path.unwrap_or("./models/face_recognition_sface_2021dec.onnx");

        if !Path::new(fd_path).exists() {
            return Err(FaceRecognitionError::ModelNotFound(fd_path.to_string()));
        }
        if !Path::new(fr_path).exists() {
            return Err(FaceRecognitionError::ModelNotFound(fr_path.to_string()));
        }

        debug!("Initializing face detection model: {}", fd_path);
        let detector = FaceDetectorYN::create(
            fd_path,
            "",
            Size::new(400, 400), // Match C++ default size
            SCORE_THRESHOLD,
            NMS_THRESHOLD,
            TOP_K,
            opencv::dnn::DNN_BACKEND_OPENCV,
            opencv::dnn::DNN_TARGET_CPU,
        )?;

        debug!("Initializing face recognition model: {}", fr_path);
        let face_recognizer = FaceRecognizerSF::create(
            fr_path,
            "",
            opencv::dnn::DNN_BACKEND_OPENCV,
            opencv::dnn::DNN_TARGET_CPU,
        )?;

        Ok(Self {
            detector,
            face_recognizer,
            max_size: max_size.unwrap_or(600),
            db_load_status: Arc::new(RwLock::new(DbLoadStatus::NotLoaded)),
            features_map: Arc::new(RwLock::new(HashMap::new())),
            db_path: Arc::new(RwLock::new(None)),
            last_mod_time: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
            watcher: Arc::new(Mutex::new(None)),
            watcher_running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn set_max_size(&mut self, size: i32) {
        self.max_size = size;
    }

    pub async fn get_db_path(&self) -> Option<PathBuf> {
        self.db_path.read().await.clone()
    }

    pub async fn set_db_path<P: AsRef<Path>>(&self, path: P) {
        let mut db_status = self.db_load_status.write().await;
        *db_status = DbLoadStatus::NotLoaded;
        drop(db_status);

        let mut db_path = self.db_path.write().await;
        *db_path = Some(path.as_ref().to_path_buf());
    }

    pub async fn load_persons_db<P: AsRef<Path>>(
        &mut self,
        persondb_folder: P,
        force: bool,
        visualize: bool,
    ) -> Result<()> {
        let path = persondb_folder.as_ref().to_path_buf();

        // Check if we need to load
        let current_path = self.db_path.read().await.clone();
        let current_status = *self.db_load_status.read().await;

        if current_path.is_none() || current_path.as_ref() != Some(&path) {
            info!("Loading persons database from: {}", path.display());
            let mut db_status = self.db_load_status.write().await;
            *db_status = DbLoadStatus::NotLoaded;
            drop(db_status);

            let mut db_path = self.db_path.write().await;
            *db_path = Some(path.clone());
        } else if current_status == DbLoadStatus::Loaded && !force {
            debug!("PersonsDB already loaded, skipping");
            return Ok(());
        }

        // Set loading status
        let mut db_status = self.db_load_status.write().await;
        *db_status = DbLoadStatus::Loading;
        drop(db_status);

        info!("Loading persons database from: {}", path.display());

        // Clear existing features
        let mut features = self.features_map.write().await;
        features.clear();
        drop(features);

        // Iterate over directories
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let person_path = entry.path();

            if person_path.is_dir() {
                let person_name = person_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                debug!("Loading person: {}", person_name);
                let mut person_features = Vec::new();

                // Load images from person directory
                for img_entry in std::fs::read_dir(&person_path)? {
                    let img_entry = img_entry?;
                    let img_path = img_entry.path();

                    if !img_path.is_dir() {
                        let filename = img_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                        // Skip visualize files
                        if filename.contains("_visualize") {
                            continue;
                        }

                        debug!(
                            "Loading image: {} for person {}",
                            img_path.display(),
                            person_name
                        );

                        let img = imread(img_path.to_str().unwrap(), IMREAD_COLOR)?;
                        if img.empty() {
                            error!("Cannot read image: {}", img_path.display());
                            continue;
                        }

                        // Extract features from all detected faces
                        let detected_faces = self.extract_features(img.clone()).await?;
                        for detected_face in detected_faces {
                            person_features.push(detected_face.feature.try_clone()?);
                        }

                        // Create visualized version if requested
                        if visualize {
                            let stem = img_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("image");
                            let extension = img_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("jpg");
                            let visualize_path =
                                person_path.join(format!("{}_visualize.{}", stem, extension));

                            let mut vis_img = img.clone();
                            let faces = self.extract_features(vis_img.clone()).await?;
                            for face in faces {
                                if let Ok(bbox) = face.bbox_scaled(vis_img.size()?) {
                                    self.visualize_face(&mut vis_img, bbox)?;
                                }
                            }

                            let _ = imwrite(
                                visualize_path.to_str().unwrap(),
                                &vis_img,
                                &opencv::core::Vector::new(),
                            );
                        }
                    }
                }

                // Store features for this person
                let mut features_map = self.features_map.write().await;
                features_map.insert(person_name, person_features);
            }
        }

        // Set loaded status
        let mut db_status = self.db_load_status.write().await;
        *db_status = DbLoadStatus::Loaded;

        info!("Database loading completed");
        Ok(())
    }

    pub async fn start_watching(&self, _check_interval_seconds: u64) -> Result<()> {
        let db_path = {
            let path_guard = self.db_path.read().await;
            path_guard
                .clone()
                .ok_or(FaceRecognitionError::DatabaseNotLoaded)?
        };

        if self
            .watcher_running
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            debug!("Watcher already running");
            return Ok(());
        }

        // Update last modification time
        let latest_mod_time = get_latest_mod_time(&db_path)?;
        let mut last_mod = self.last_mod_time.write().await;
        *last_mod = latest_mod_time;
        drop(last_mod);

        // Start file watcher
        let mut watcher_guard = self.watcher.lock().unwrap();
        let mut watcher = FolderWatcher::new()?;
        watcher.start_watching(&db_path)?;

        // Store watcher before moving it
        *watcher_guard = Some(watcher);
        drop(watcher_guard);
        self.watcher_running
            .store(true, std::sync::atomic::Ordering::Relaxed);

        info!("Started watching database folder: {}", db_path.display());
        Ok(())
    }

    pub async fn stop_watching(&self) {
        let mut watcher_guard = self.watcher.lock().unwrap();
        if let Some(mut watcher) = watcher_guard.take() {
            watcher.stop_watching();
        }
        self.watcher_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
        info!("Stopped watching database folder");
    }

    async fn extract_features(&mut self, mut frame: Mat) -> Result<Vec<DetectedFace>> {
        if frame.empty() {
            return Err(FaceRecognitionError::InvalidImage);
        }

        let original_size = frame.size()?;
        self.resize_frame(&mut frame, true)?;

        debug!("Frame size: {}x{}", frame.cols(), frame.rows());

        // Set detector input size to match the resized frame (like C++ version)
        let frame_size = frame.size()?;
        self.detector.set_input_size(frame_size)?;

        // Detect faces directly on the resized frame
        let mut faces = Mat::default();
        match self.detector.detect(&frame, &mut faces) {
            Ok(_) => {}
            Err(e) => {
                error!("Face detection failed: {}", e);
                return Err(FaceRecognitionError::DetectionFailed);
            }
        }

        debug!("Found {} faces", faces.rows());

        if faces.rows() <= 0 {
            warn!("Cannot find any faces");
            return Ok(Vec::new());
        }

        let mut detected_faces = Vec::new();
        for i in 0..faces.rows() {
            let face_row = faces.row(i)?;

            // Use face detection results directly - no coordinate scaling needed
            // since detector input size matches frame size
            let mut aligned_img = Mat::default();
            match self
                .face_recognizer
                .align_crop(&frame, &face_row, &mut aligned_img)
            {
                Ok(_) => {}
                Err(e) => {
                    debug!("Failed to align/crop face {}: {}", i, e);
                    continue;
                }
            }

            // Extract features
            let mut feature = Mat::default();
            match self.face_recognizer.feature(&aligned_img, &mut feature) {
                Ok(_) => {
                    debug!(
                        "Feature extraction successful for face {}, feature size: {}x{}",
                        i,
                        feature.rows(),
                        feature.cols()
                    );
                    if feature.rows() > 0 && feature.cols() > 0 {
                        let first_few: Vec<f32> = (0..std::cmp::min(5, feature.cols()))
                            .map(|j| *feature.at_2d::<f32>(0, j).unwrap_or(&0.0))
                            .collect();
                        debug!("First 5 feature values: {:?}", first_few);
                    }
                }
                Err(e) => {
                    debug!("Failed to extract features for face {}: {}", i, e);
                    continue;
                }
            }

            detected_faces.push(DetectedFace::new_with_detection_size(
                "Unknown".to_string(),
                face_row.try_clone()?,
                feature.try_clone()?,
                original_size,
                frame.size()?, // Current resized frame size
            ));
        }

        Ok(detected_faces)
    }

    fn resize_frame(&self, frame: &mut Mat, keep_aspect_ratio: bool) -> Result<()> {
        if self.max_size <= 0 {
            return Ok(()); // No resizing requested
        }

        if frame.empty() {
            return Err(FaceRecognitionError::InvalidImage);
        }

        let cols = frame.cols();
        let rows = frame.rows();

        if keep_aspect_ratio {
            if cols > self.max_size || rows > self.max_size {
                let max_dim = std::cmp::max(cols, rows);
                let scale = self.max_size as f64 / max_dim as f64;
                let new_size =
                    Size::new((cols as f64 * scale) as i32, (rows as f64 * scale) as i32);
                let mut resized = Mat::default();
                opencv::imgproc::resize(
                    frame,
                    &mut resized,
                    new_size,
                    0.0,
                    0.0,
                    opencv::imgproc::INTER_LINEAR,
                )?;
                *frame = resized;
            }
        } else {
            let new_size = Size::new(self.max_size, self.max_size);
            let mut resized = Mat::default();
            opencv::imgproc::resize(
                frame,
                &mut resized,
                new_size,
                0.0,
                0.0,
                opencv::imgproc::INTER_LINEAR,
            )?;
            *frame = resized;
        }

        Ok(())
    }

    fn visualize_face(&self, frame: &mut Mat, bbox: Rect2i) -> Result<()> {
        let color = Scalar::new(0.0, 255.0, 0.0, 0.0); // Green
        rectangle(frame, bbox, color, 2, LINE_8, 0)?;
        Ok(())
    }

    async fn find_best_match(
        &mut self,
        face_feature: &Mat,
        threshold: f32,
    ) -> Result<MatchResults> {
        let features_map = self.features_map.read().await;

        let mut results = Vec::new();
        let mut best_match = MatchResult::new("Unknown".to_string(), 0.0);

        for (person_name, features) in features_map.iter() {
            for (feature_idx, feature) in features.iter().enumerate() {
                let score = self.face_recognizer.match_(
                    face_feature,
                    feature,
                    opencv::objdetect::FaceRecognizerSF_DisType::FR_COSINE as i32,
                )? as f32;
                results.push(MatchResult::new(person_name.clone(), score));

                // Debug feature comparison
                if feature_idx == 0 {
                    // Only debug the first feature per person to avoid spam
                    let query_first_5: Vec<f32> = (0..5)
                        .map(|j| *face_feature.at_2d::<f32>(0, j).unwrap_or(&0.0))
                        .collect();
                    let db_first_5: Vec<f32> = (0..5)
                        .map(|j| *feature.at_2d::<f32>(0, j).unwrap_or(&0.0))
                        .collect();
                    debug!(
                        "Person {}, feature #{}, score: {}",
                        person_name, feature_idx, score
                    );
                    debug!("  Query: {:?}", query_first_5);
                    debug!("  DB:    {:?}", db_first_5);
                } else {
                    debug!(
                        "Person {}, feature #{}, score: {}",
                        person_name, feature_idx, score
                    );
                }

                if score > best_match.score && score > threshold {
                    best_match = MatchResult::new(person_name.clone(), score);
                }
            }
        }

        Ok(MatchResults {
            results,
            best_match,
        })
    }

    pub async fn run(
        &mut self,
        frame: &mut Mat,
        threshold: f32,
        visualize: bool,
    ) -> Result<Vec<MatchResult>> {
        let frame_for_detection = if visualize {
            frame.clone()
        } else {
            frame.clone()
        };

        let detected_faces = self.extract_features(frame_for_detection).await?;
        let mut results = Vec::new();

        for (i, face) in detected_faces.iter().enumerate() {
            let match_results = self.find_best_match(&face.feature, threshold).await?;
            let best = match_results.best_match;

            info!("Face {} best match: {}", i + 1, best.name);
            results.push(best.clone());

            if visualize {
                // Scale bounding box to match the visualization frame size
                if let Ok(bbox) = face.bbox_scaled(frame.size()?) {
                    self.visualize_face(frame, bbox)?;
                    self.annotate_with_name_scaled(frame, &face, &best.name)?;
                }
            }
        }

        Ok(results)
    }

    pub async fn run_one_face(
        &mut self,
        mut frame: Mat,
        threshold: f32,
        visualize: bool,
    ) -> Result<MatchResult> {
        let results = self.run(&mut frame, threshold, visualize).await?;

        if results.is_empty() {
            return Ok(MatchResult::new("Unknown".to_string(), 0.0));
        }

        let mut best_match = &results[0];
        for result in &results {
            if result.score > best_match.score {
                best_match = result;
            }
        }

        Ok(best_match.clone())
    }

    #[allow(dead_code)]
    fn annotate_with_name(&self, frame: &mut Mat, face: &DetectedFace, name: &str) -> Result<()> {
        let bbox = face.bbox()?;

        // Text parameters
        let font_face = FONT_HERSHEY_SIMPLEX;
        let font_scale = 0.8;
        let thickness = 2;
        let mut baseline = 0;

        let text_size = get_text_size(name, font_face, font_scale, thickness, &mut baseline)?;
        let text_x = bbox.x + (bbox.width - text_size.width) / 2;
        let text_y = std::cmp::max(bbox.y - text_size.height - 5, 0);

        // Draw background rectangle
        let bg_rect = Rect2i::new(
            text_x - 2,
            text_y - 2,
            text_size.width + 4,
            text_size.height + 4,
        );

        let bg_color = Scalar::new(0.0, 0.0, 0.0, 0.0); // Black background
        rectangle(frame, bg_rect, bg_color, -1, LINE_8, 0)?;

        // Draw text
        let text_color = Scalar::new(255.0, 255.0, 255.0, 0.0); // White text
        let text_pos = Point::new(text_x, text_y + text_size.height);
        put_text(
            frame, name, text_pos, font_face, font_scale, text_color, thickness, LINE_8, false,
        )?;

        Ok(())
    }

    fn annotate_with_name_scaled(
        &self,
        frame: &mut Mat,
        face: &DetectedFace,
        name: &str,
    ) -> Result<()> {
        let bbox = face.bbox_scaled(frame.size()?)?;

        // Text parameters - scale font based on image size
        let font_face = FONT_HERSHEY_SIMPLEX;
        let base_font_scale = 0.8;
        // Scale font based on image width - larger images get bigger text
        let font_scale = base_font_scale * (frame.cols() as f64 / 800.0).max(0.5).min(3.0);
        let thickness = ((frame.cols() as f64 / 800.0).max(1.0).min(4.0)) as i32;
        let mut baseline = 0;

        let text_size = get_text_size(name, font_face, font_scale, thickness, &mut baseline)?;
        let text_x = bbox.x + (bbox.width - text_size.width) / 2;
        let text_y = std::cmp::max(bbox.y - text_size.height - 5, 0);

        // Draw background rectangle
        let bg_rect = Rect2i::new(
            text_x - 2,
            text_y - 2,
            text_size.width + 4,
            text_size.height + 4,
        );

        let bg_color = Scalar::new(0.0, 0.0, 0.0, 0.0); // Black background
        rectangle(frame, bg_rect, bg_color, -1, LINE_8, 0)?;

        // Draw text
        let text_color = Scalar::new(255.0, 255.0, 255.0, 0.0); // White text
        let text_pos = Point::new(text_x, text_y + text_size.height);
        put_text(
            frame, name, text_pos, font_face, font_scale, text_color, thickness, LINE_8, false,
        )?;

        Ok(())
    }

    /// Simple face detection only (no recognition) - returns count of detected faces
    pub async fn detect_faces_count<P: AsRef<Path>>(&mut self, image_path: P) -> Result<usize> {
        let frame = imread(image_path.as_ref().to_str().unwrap(), IMREAD_COLOR)?;
        if frame.empty() {
            return Err(FaceRecognitionError::InvalidImage);
        }

        let detected_faces = self.extract_features(frame).await?;
        Ok(detected_faces.len())
    }
}
