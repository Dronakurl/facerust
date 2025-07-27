# 🦀 FaceRust DeepStream Integration Guide

This guide shows you how to integrate the Rust `facerust` library into your existing C++/DeepStream application.

## 🎯 **Why This Approach?**

✅ **Keep your existing DeepStream pipeline intact**  
✅ **No major C++ code restructuring needed**  
✅ **Drop-in replacement for your current face recognition**  
✅ **All the benefits of Rust: memory safety, performance, async**  

## 📋 **Integration Steps**

### **Step 1: Build the Rust Library**

```bash
cd facerust
cargo build --release
```

This creates:
- `target/release/libfacerust.a` (static library)
- `target/release/libfacerust.so` (dynamic library) 

### **Step 2: Copy Files to Your Project**

Copy these files to your DeepStream project directory:
```bash
cp facerust.h /path/to/your/deepstream/project/include/
cp facerust_wrapper.hpp /path/to/your/deepstream/project/include/
cp target/release/libfacerust.so /path/to/your/deepstream/project/lib/
# OR for static linking:
cp target/release/libfacerust.a /path/to/your/deepstream/project/lib/
```

### **Step 3: Modify Your CMakeLists.txt or Makefile**

**For CMake:**
```cmake
# Add to your CMakeLists.txt
find_library(FACERUST_LIB facerust PATHS ${PROJECT_SOURCE_DIR}/lib)
target_link_libraries(your_target ${FACERUST_LIB} pthread dl)
target_include_directories(your_target PRIVATE ${PROJECT_SOURCE_DIR}/include)
```

**For Makefile:**
```makefile
# Add to your LDFLAGS
LDFLAGS += -L./lib -lfacerust -lpthread -ldl
```

### **Step 4: Modify Your Code**

#### **A. Update `grid.hpp` (or your config header):**

```cpp
#include "facerust_wrapper.hpp"

struct CustomData {
    // ... your existing fields ...
    
    // Add Rust face recognizer
    std::unique_ptr<FaceRecognitionRust> facerecognizer_rust;
    bool facerecognizer_rust_on = false;  // Add this to your config
    
    // ... rest of your fields ...
};
```

#### **B. Update `main.cpp` initialization:**

Replace this section:
```cpp
// OLD CODE:
if (data.facerecognizer_on) {
    auto facerecognizer = make_shared<FaceRecognition>(/* ... */);
    data.facerecognizer = facerecognizer;
    data.facerecognizer->loadPersonsDB(data.persondb_folder);
    gst_pad_add_probe(osdpad, GST_PAD_PROBE_TYPE_BUFFER, extract_probe, &data, NULL);
}
```

With this:
```cpp
// NEW CODE:
if (data.facerecognizer_rust_on) {
    if (!data.tracker_on) {
        g_error("Tracker must be on for face recognition");
        return -1;
    }
    
    try {
        data.facerecognizer_rust = std::make_unique<FaceRecognitionRust>();
        data.facerecognizer_rust->loadPersonsDB(data.persondb_folder);
        
        g_info("Rust face recognition initialized successfully");
        gst_pad_add_probe(osdpad, GST_PAD_PROBE_TYPE_BUFFER, extract_probe, &data, NULL);
    } catch (const std::exception& e) {
        g_error("Failed to initialize Rust face recognition: %s", e.what());
        return -1;
    }
}
```

#### **C. Update `extract_probe.cpp`:**

Replace this line:
```cpp
// OLD CODE:
objData.match = config->facerecognizer->run_one_face(object_img, 0.3);
```

With this:
```cpp
// NEW CODE:
try {
    FaceRecognitionRust* rust_face_rec = config->facerecognizer_rust.get();
    objData.match = rust_face_rec->run_one_face(object_img, 0.3f);
    DEBUG("Running Rust face recognition on %s -> %s", obj_meta->obj_label,
          objData.match.toString().c_str());
} catch (const std::exception& e) {
    g_warning("Rust face recognition failed: %s", e.what());
    objData.match = MatchResult("unknown", 0.0f);
}
```

## 🛠️ **Build and Test**

1. **Build your project as usual**
2. **Models are automatically downloaded during build:**
   ```bash
   cd facerust
   cargo build --release  # Downloads ONNX models automatically
   ```
   
   Models will be in `facerust/models/` and copied to your project during integration.

3. **Test with your existing pipeline**

## ⚡ **Performance Notes**

- **First recognition call**: May be slower due to Rust/Tokio initialization
- **Subsequent calls**: Should be as fast or faster than C++
- **Memory usage**: Similar to C++ version
- **Thread safety**: The wrapper handles thread safety internally

## 🐛 **Debugging**

If you encounter issues:

1. **Check library loading:**
   ```bash
   ldd your_executable | grep facerust
   ```

2. **Enable debug logging:**
   ```bash
   RUST_LOG=debug ./your_executable
   ```

3. **Verify models exist:**
   ```bash
   ls -la /app/models/face_*.onnx
   ```

## 🔄 **Gradual Migration**

You can run **both** face recognition systems in parallel:

1. Keep your existing C++ face recognition code
2. Add the Rust version with a different config flag
3. Compare results during testing
4. Switch over when confident

## 🚀 **Benefits You'll Get**

- ✅ **Better memory safety** (no more segfaults from face recognition)
- ✅ **Improved error handling** (graceful failures instead of crashes)  
- ✅ **Better coordinate scaling** (bounding boxes positioned correctly)
- ✅ **Async database loading** (non-blocking operations)
- ✅ **Robust file watching** (automatic database updates)
- ✅ **Same API compatibility** (`MatchResult` works the same)

---

**Need help?** The Rust face recognition library is now a drop-in replacement for your C++ version! 🎉