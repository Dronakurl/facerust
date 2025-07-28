# FaceRust C Integration

This directory contains the C integration example for FaceRust, demonstrating how to use the Rust face recognition library from C code.

## Quick Start

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config 
# Install opencv version 4.12+
./install_opencv.sh
```

### Building and Running

```bash
# Simple build (dynamic linking - recommended)
./build.sh

# Clean build with test
./build.sh --clean --test

# Static linking (experimental)
./build.sh --static

# Generate compile_commands.json for language servers (clangd, etc.)
./build.sh --bear
```

### Manual CMake Build

```bash
# Dynamic linking (default)
mkdir build && cd build
cmake ..
cmake --build .

# Run from facerust root directory (required for model paths)
cd .. && LD_LIBRARY_PATH=target/release ./c_integration/build/face_demo media/db

# Or use the test target (automatically runs from correct directory)
cd build && cmake --build . --target test_demo

# Static linking
mkdir build && cd build
cmake -DUSE_STATIC_LINKING=ON ..
cmake --build .
cd .. && ./c_integration/build/face_demo_static media/db
```

## Files

- `example_c.cpp` - Main C demonstration program (uses OpenCV for real image loading)
- `CMakeLists.txt` - CMake build configuration
- `build.sh` - Convenient build script
- `README.md` - This file
- `Makefile` - Legacy Makefile (kept for reference)

## Library Linking

### Dynamic Linking (Recommended)

- **Pros**: Simpler build process, smaller executable
- **Cons**: Requires `LD_LIBRARY_PATH` to be set
- **Usage**: `cd .. && LD_LIBRARY_PATH=target/release ./c_integration/build/face_demo media/db`

### Static Linking (Experimental)

- **Pros**: Self-contained executable, no LD_LIBRARY_PATH needed
- **Cons**: Larger executable, requires OpenCV development libraries
- **Usage**: `cd .. && ./c_integration/build/face_demo_static media/db`

## API Usage

The C API provides these main functions:

```c
// Create face recognition instance
CFaceRecognition* facerecognition_create();

// Load persons database
int facerecognition_load_persons_db(CFaceRecognition* face_rec, const char* db_path);

// Run face recognition on image data
CMatchResult facerecognition_run_one_face_opencv_mat(
    CFaceRecognition* face_rec, 
    unsigned char* image_data, 
    int rows, int cols, int channels, 
    float threshold
);

// Clean up
void facerecognition_free_match_result(CMatchResult* result);
void facerecognition_destroy(CFaceRecognition* face_rec);
```

## Image Format

The `facerecognition_run_one_face_opencv_mat` function expects:

- **Format**: BGR or RGB pixel data
- **Layout**: Row-major order (height × width × channels)
- **Channels**: Typically 3 for color images
- **Data type**: `unsigned char` array

## Language Server Support

For enhanced development experience with IntelliSense, autocomplete, and error checking:

### Using Bear (Recommended)

```bash
# Install bear (Ubuntu/Debian)
sudo apt-get install bear

# Generate compile_commands.json
./build.sh --bear
```

This creates a `compile_commands.json` file that most language servers (clangd, ccls, etc.) can use for full C IntelliSense support.

### CMake Native Support

CMake automatically generates `compile_commands.json` when `CMAKE_EXPORT_COMPILE_COMMANDS` is enabled (already configured in this project).

## Notes

- ONNX models are automatically downloaded during the Rust build process
- The example loads the real image `media/IMG.jpg` using OpenCV for actual face recognition
- The demo must be run from the facerust root directory for proper model and image path resolution
- The `compile_commands.json` file enables full language server support for C development

