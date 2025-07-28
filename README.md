# 🦀 FaceRust - Face Recognition in Rust

Fast face recognition library with CLI tool and C/C++ integration.

## 🚀 Quick Start

```bash
# Build (automatically downloads ONNX models)
cargo build --release

# Run face recognition
cargo run --bin facerust-cli -- -i image.jpg -d ./media/db
```

**Models:** ONNX files are downloaded automatically during build via `build.rs`

**Database structure:** Put person photos in folders named after them:
```
media/db/
├── john/
│   └── photo.jpg
└── jane/
    └── image.jpg
```

## 🔧 Rust Library

```rust
use facerust::FaceRecognition;

let mut face_rec = FaceRecognition::new(None, None, None)?;
face_rec.load_persons_db("./media/db", false, false).await?;
let results = face_rec.run(&mut image, 0.4, true).await?;
```

## 🔗 C/C++ Integration

```c
#include "facerust.h"

CFaceRecognition* face_rec = facerecognition_create();
facerecognition_load_persons_db(face_rec, "./media/db");
CMatchResult result = facerecognition_run_one_face_opencv_mat(
    face_rec, image_data, height, width, channels, 0.3f);
printf("Recognized: %s\n", result.name);
```

**Quick start:**
```bash
cd c_integration
./build.sh --clean --test
```

**Features:** CMake build system • Static/dynamic linking • Language server support (`--bear`)

**Documentation:** [`c_integration/README.md`](./c_integration/README.md)

## 🛠️ Requirements

- Rust 1.70+
- OpenCV 4.11.0+ (install via package manager: `sudo apt-get install libopencv-dev`)
- For C integration: CMake 3.16+, GCC/Clang
- Optional: `bear` for language server support

---

Features: YuNet face detection • SFace recognition • File watching • CLI tool • C/C++ interface