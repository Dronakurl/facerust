# FaceRust C Integration Example

This directory contains a complete example of how to integrate the Rust `facerust` library into a C application.

## 📁 Files

- `example_c_integration.c` - Simple C program demonstrating the integration
- `facerust.h` - C header file with function declarations
- `Makefile` - Build configuration for the demo
- `src/ffi.rs` - Rust FFI implementation (C interface)

## 🚀 Quick Start

### 1. Build and Run

```bash
# Build the demo
make

# Run with the sample database
make test

# Or run manually
LD_LIBRARY_PATH=./target/release ./face_demo ./media/db
```

### 2. Expected Output

```
=== FaceRust C Integration Demo ===
Database path: ./media/db

1. Creating FaceRecognition instance...
✓ FaceRecognition instance created successfully

2. Loading persons database from ./media/db...
✓ Persons database loaded successfully

3. Creating test image data...
✓ Created test image: 480x640 with 3 channels

4. Running face recognition...

5. Face recognition results:
   Name: unknown
   Score: 0.000
   Threshold: 0.300
⚠ No face recognized above threshold (got 0.0%, need 30.0%)

6. Cleaning up...
✓ Cleanup completed

=== Demo completed successfully! ===
```

*Note: The demo uses dummy image data, so "unknown" result is expected.*

## 🔧 Integration Steps

### 1. **Include the Header**
```c
#include "facerust.h"
```

### 2. **Create Face Recognition Instance**
```c
CFaceRecognition* face_rec = facerecognition_create();
if (face_rec == NULL) {
    // Handle error - models not found
}
```

### 3. **Load Database**
```c
int result = facerecognition_load_persons_db(face_rec, "/path/to/db");
if (result != 0) {
    // Handle error - database load failed
}
```

### 4. **Run Recognition**
```c
CMatchResult result = facerecognition_run_one_face_opencv_mat(
    face_rec,
    image_data,    // Raw BGR/RGB pixel data
    height,        // Image height
    width,         // Image width  
    channels,      // 1 or 3 channels
    0.3f          // Recognition threshold
);

printf("Recognized: %s (score: %.3f)\n", result.name, result.score);
```

### 5. **Cleanup**
```c
facerecognition_free_match_result(&result);
facerecognition_destroy(face_rec);
```

## 📋 Requirements

### System Dependencies
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libopencv-dev

# Or use the Makefile
make install-deps
```

### Model Files
ONNX models are **automatically downloaded** during `cargo build`. If needed manually:
```bash
./download_models.sh
# or
make download-models
```

Required files in `models/` directory:
- `models/face_detection_yunet_2023mar.onnx`
- `models/face_recognition_sface_2021dec.onnx`

### Database Structure
```
media/db/
├── person1/
│   ├── photo1.jpg
│   └── photo2.jpg
├── person2/
│   └── photo1.jpg
└── ...
```

## 🔗 Linking

### Static Linking
```makefile
LDFLAGS = -L./target/release -lfacerust -lpthread -ldl -lm
```

### Dynamic Linking
```bash
# Set library path
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH

# Or use rpath during compilation
gcc -Wl,-rpath,./target/release -L./target/release -lfacerust ...
```

## 🐛 Troubleshooting

### "Failed to create FaceRecognition instance"
- Check that ONNX model files exist in `models/` directory
- Verify OpenCV is properly installed

### "Failed to load persons database"
- Ensure database path exists and contains person folders
- Check folder permissions

### "Library not found" errors
- Set `LD_LIBRARY_PATH` to include `./target/release`
- Verify `libfacerust.so` was built successfully

### Debugging
```bash
# Enable Rust logging
RUST_LOG=debug LD_LIBRARY_PATH=./target/release ./face_demo ./media/db

# Check library dependencies
ldd ./face_demo
```

## 🔄 Real World Usage

In a real application, you would:

1. **Load actual images** using OpenCV, STBI, or similar:
   ```c
   // Using OpenCV (C API)
   IplImage* img = cvLoadImage("photo.jpg", CV_LOAD_IMAGE_COLOR);
   unsigned char* data = (unsigned char*)img->imageData;
   ```

2. **Convert pixel formats** if needed (BGR ↔ RGB)

3. **Handle errors** properly with retry logic

4. **Use threading** to avoid blocking the main thread

5. **Cache instances** instead of creating/destroying repeatedly

## 🚀 Performance Tips

- **Reuse the FaceRecognition instance** - creation is expensive
- **Load database once** at startup
- **Use appropriate thresholds** (0.3-0.6 typically work well)
- **Preprocess images** for better recognition (resize, normalize)

---

This example demonstrates the complete integration flow. The C interface provides a simple, safe way to use Rust face recognition in existing C/C++ applications! 🎉