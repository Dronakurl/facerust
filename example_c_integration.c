#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "facerust.h"

// Simple function to load a simple test image (just create dummy data)
// In a real application, you would use OpenCV or another image library
void create_dummy_image_data(unsigned char** data, int* rows, int* cols, int* channels) {
    *rows = 480;
    *cols = 640;
    *channels = 3;
    
    // Allocate memory for BGR image data
    int size = (*rows) * (*cols) * (*channels);
    *data = (unsigned char*)malloc(size);
    
    // Fill with a simple pattern (this won't be a real face, just for testing the interface)
    for (int i = 0; i < size; i++) {
        (*data)[i] = (unsigned char)(i % 256);
    }
}

int main(int argc, char* argv[]) {
    printf("=== FaceRust C Integration Demo ===\n");
    
    // Check command line arguments
    if (argc != 2) {
        printf("Usage: %s <database_path>\n", argv[0]);
        printf("Example: %s ./media/db\n", argv[0]);
        return 1;
    }
    
    const char* db_path = argv[1];
    printf("Database path: %s\n", db_path);
    
    // Step 1: Create FaceRecognition instance
    printf("\n1. Creating FaceRecognition instance...\n");
    CFaceRecognition* face_rec = facerecognition_create();
    if (face_rec == NULL) {
        printf("ERROR: Failed to create FaceRecognition instance!\n");
        printf("Make sure the ONNX model files exist in the models/ directory:\n");
        printf("  - models/face_detection_yunet_2023mar.onnx\n");
        printf("  - models/face_recognition_sface_2021dec.onnx\n");
        return 1;
    }
    printf("✓ FaceRecognition instance created successfully\n");
    
    // Step 2: Load persons database
    printf("\n2. Loading persons database from %s...\n", db_path);
    int load_result = facerecognition_load_persons_db(face_rec, db_path);
    if (load_result != 0) {
        printf("ERROR: Failed to load persons database!\n");
        printf("Make sure the database directory exists and contains person folders\n");
        facerecognition_destroy(face_rec);
        return 1;
    }
    printf("✓ Persons database loaded successfully\n");
    
    // Step 3: Create dummy image data (in a real app, you'd load an actual image)
    printf("\n3. Creating test image data...\n");
    unsigned char* image_data;
    int rows, cols, channels;
    create_dummy_image_data(&image_data, &rows, &cols, &channels);
    printf("✓ Created test image: %dx%d with %d channels\n", rows, cols, channels);
    
    // Step 4: Run face recognition
    printf("\n4. Running face recognition...\n");
    float threshold = 0.3f;
    CMatchResult result = facerecognition_run_one_face_opencv_mat(
        face_rec, image_data, rows, cols, channels, threshold
    );
    
    // Step 5: Display results
    printf("\n5. Face recognition results:\n");
    printf("   Name: %s\n", result.name ? result.name : "NULL");
    printf("   Score: %.3f\n", result.score);
    printf("   Threshold: %.3f\n", threshold);
    
    if (result.score >= threshold) {
        printf("✓ Face recognized as: %s (confidence: %.1f%%)\n", 
               result.name, result.score * 100);
    } else {
        printf("⚠ No face recognized above threshold (got %.1f%%, need %.1f%%)\n", 
               result.score * 100, threshold * 100);
    }
    
    // Step 6: Cleanup
    printf("\n6. Cleaning up...\n");
    facerecognition_free_match_result(&result);
    facerecognition_destroy(face_rec);
    free(image_data);
    printf("✓ Cleanup completed\n");
    
    printf("\n=== Demo completed successfully! ===\n");
    printf("\nNote: This demo used dummy image data. In a real application:\n");
    printf("- Load actual images using OpenCV, STBI, or similar libraries\n");
    printf("- Pass the raw BGR/RGB pixel data to facerecognition_run_one_face_opencv_mat()\n");
    printf("- The function expects row-major order pixel data (height × width × channels)\n");
    
    return 0;
}