# 🦀 FaceRust - Face Recognition in Rust

> A blazingly fast face recognition library written in Rust 🚀

## ✨ Features

- 🎯 **Real-time face detection** using OpenCV's YuNet model
- 🧠 **Face recognition** with SFace deep learning model  
- 📁 **Smart database management** with automatic file watching
- ⚡ **Async-first design** for maximum performance
- 🔄 **Hot reloading** - database changes detected automatically
- 🖼️ **Image visualization** with bounding boxes and labels
- 🛠️ **CLI tool** ready to use out of the box

## 🚀 Quick Start

```bash
# Clone and build
git clone <repo>
cd facerust
cargo build --release

# Run face recognition on an image
cargo run --bin facerust-cli -- -i /path/to/image.jpg -d /path/to/faces_db

# Test database hot-reloading
cargo run --bin facerust-cli -- -i image.jpg -d faces_db --test-mode
```

## 📁 Database Structure

```
faces_db/
├── person1/
│   ├── photo1.jpg
│   ├── photo2.jpg
│   └── photo3.jpg
└── person2/
    ├── image1.jpg
    └── image2.jpg
```

## 🎛️ CLI Options

```bash
Face Recognition CLI Tool

Options:
  -i, --image <FILE>  Path to the input image
  -d, --db <DIR>      Path to the faces database  
  -t, --test-mode     Test database update mechanism
  -h, --help          Show help
```

## 🔧 Library Usage

```rust
use facerust::FaceRecognition;

// Initialize with ONNX models
let mut face_rec = FaceRecognition::new(
    Some("models/face_detection_yunet_2023mar.onnx"),
    Some("models/face_recognition_sface_2021dec.onnx"),
    Some(1000)
)?;

// Load person database
face_rec.load_persons_db("./faces_db", false, false).await?;

// Start watching for database changes
face_rec.start_watching(5).await?;

// Run face recognition
let results = face_rec.run(&mut image, 0.4, true).await?;
```

## 🎯 What's Inside

- 📦 **Core Library** - Face recognition engine
- 🖥️ **CLI Tool** - Ready-to-use command line interface  
- 🔍 **File Watcher** - Automatic database reload on changes
- 📊 **Rich Types** - Structured data for matches and results
- ⚡ **Async Runtime** - Built on Tokio for performance

## 🛠️ Requirements

- Rust 1.70+
- OpenCV 4.11.0+ (required for YuNet ONNX model compatibility)
- pkg-config

### 📦 OpenCV Installation

The face recognition functionality requires **OpenCV 4.11.0 or newer** for proper ONNX model support. Most system packages provide older versions that won't work with the YuNet model.

**Quick Installation:**
```bash
# Run the provided installation script
./install_opencv.sh
```

**Manual Installation:**
If you prefer to install manually, the script builds OpenCV 4.11.0 from source with the required modules. This takes 15-30 minutes but ensures compatibility.

**Troubleshooting:**
- If you get "Layer with requested id=-1 not found" errors, you need to upgrade OpenCV
- Ubuntu/Debian system packages (libopencv-dev) are typically too old (4.6.0)
- The installation script handles all dependencies automatically

## 📈 Performance

Built with Rust's zero-cost abstractions and OpenCV's optimized computer vision algorithms for maximum speed and efficiency.

---

*Made with ❤️ and lots of ☕*