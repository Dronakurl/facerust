// Add this to your grid.hpp or create a new header

#pragma once

#include "facerust_wrapper.hpp"  // Include the Rust wrapper
#include <memory>

// Add this field to your CustomData struct
struct CustomData {
    // ... your existing fields ...
    
    // Replace or add alongside your existing face recognizer
    std::unique_ptr<FaceRecognitionRust> facerecognizer_rust;
    bool facerecognizer_rust_on = false;  // Add this flag to config
    
    // ... rest of your fields ...
};

// Modified initialization in main.cpp would look like:
/*
if (data.facerecognizer_rust_on) {
    if (!data.tracker_on) {
        g_error("Tracker must be on for face recognition");
        return -1;
    }
    
    try {
        // Create Rust face recognizer
        data.facerecognizer_rust = std::make_unique<FaceRecognitionRust>();
        data.facerecognizer_rust->loadPersonsDB(data.persondb_folder);
        
        g_info("Rust face recognition initialized successfully");
        
        // Add the probe for Rust face recognition
        gst_pad_add_probe(osdpad, GST_PAD_PROBE_TYPE_BUFFER, extract_probe_rust, &data, NULL);
    } catch (const std::exception& e) {
        g_error("Failed to initialize Rust face recognition: %s", e.what());
        return -1;
    }
}
*/