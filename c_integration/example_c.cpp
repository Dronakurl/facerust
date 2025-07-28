#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "facerust.h"

// We'll use OpenCV for proper image loading
#include <opencv2/opencv.hpp>
#include <opencv2/imgcodecs.hpp>
#include <opencv2/imgproc.hpp>

// Helper structure to hold image data
typedef struct {
    int width, height, channels;
    unsigned char* data;
} SimpleImage;

// Load actual image using OpenCV
SimpleImage* load_real_image(const char* filename) {
    if (!filename) {
        printf("❌ No image filename provided\n");
        return NULL;
    }
    
    printf("📷 Loading image: %s\n", filename);
    
    // Load image using OpenCV
    cv::Mat img = cv::imread(filename, cv::IMREAD_COLOR);
    if (img.empty()) {
        printf("❌ Failed to load image: %s\n", filename);
        printf("   Make sure the file exists and is a valid image format\n");
        return NULL;
    }
    
    // OpenCV loads as BGR, which is what facerust expects
    SimpleImage* result = (SimpleImage*)malloc(sizeof(SimpleImage));
    result->width = img.cols;
    result->height = img.rows;
    result->channels = img.channels();
    
    // Allocate and copy image data
    int size = result->width * result->height * result->channels;
    result->data = (unsigned char*)malloc(size);
    memcpy(result->data, img.data, size);
    
    printf("✓ Loaded image: %dx%d with %d channels\n", 
           result->width, result->height, result->channels);
    
    return result;
}

void free_simple_image(SimpleImage* img) {
    if (img) {
        free(img->data);
        free(img);
    }
}

int main(int argc, char* argv[]) {
    printf("=== FaceRust C Integration Demo ===\n");
    
    // Check command line arguments
    if (argc < 2 || argc > 3) {
        printf("Usage: %s <database_path> [image_path]\n", argv[0]);
        printf("Examples:\n");
        printf("  %s ../media/db                    # Use default test image\n", argv[0]);
        printf("  %s ../media/db ../media/IMG.jpg   # Use specific image\n", argv[0]);
        return 1;
    }
    
    const char* db_path = argv[1];
    // Default to the provided test image if no image path is given
    const char* image_path = (argc == 3) ? argv[2] : "media/IMG.jpg";
    
    printf("Database path: %s\n", db_path);
    printf("Image path: %s\n", image_path);
    
    // Step 1: Create FaceRecognition instance
    printf("\n1. Creating FaceRecognition instance...\n");
    CFaceRecognition* face_rec = facerecognition_create();
    if (face_rec == NULL) {
        printf("ERROR: Failed to create FaceRecognition instance!\n");
        printf("Make sure the ONNX model files exist in the models/ directory:\n");
        printf("  - models/face_detection_yunet_2023mar.onnx\n");
        printf("  - models/face_recognition_sface_2021dec.onnx\n");
        printf("These models are automatically downloaded during 'cargo build'\n");
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
    
    // Step 3: Load real image data
    printf("\n3. Loading image data...\n");
    SimpleImage* img = load_real_image(image_path);
    if (img == NULL) {
        printf("ERROR: Failed to load image data\n");
        facerecognition_destroy(face_rec);
        return 1;
    }
    
    // Step 4: Run face recognition with multiple thresholds
    printf("\n4. Running face recognition tests...\n");
    
    float thresholds[] = {0.1f, 0.3f, 0.5f, 0.7f, 0.9f};
    int num_thresholds = sizeof(thresholds) / sizeof(thresholds[0]);
    
    printf("Testing recognition with different threshold values:\n");
    for (int i = 0; i < num_thresholds; i++) {
        float threshold = thresholds[i];
        printf("\n   Threshold %.1f:\n", threshold);
        
        CMatchResult result = facerecognition_run_one_face_opencv_mat(
            face_rec, img->data, img->height, img->width, img->channels, threshold
        );
        
        printf("     Name: %s\n", result.name ? result.name : "(none)");
        printf("     Score: %.3f\n", result.score);
        
        if (result.name && result.score >= threshold) {
            printf("     ✓ Match found: '%s' (confidence: %.1f%%)\n", 
                   result.name, result.score * 100);
        } else {
            printf("     ⚠ No match above threshold (got %.1f%%, need %.1f%%)\n", 
                   result.score * 100, threshold * 100);
        }
        
        facerecognition_free_match_result(&result);
    }
    
    // Step 5: Cleanup
    printf("\n5. Cleaning up...\n");
    facerecognition_destroy(face_rec);
    free_simple_image(img);
    printf("✓ Cleanup completed\n");
    
    printf("\n=== Demo completed successfully! ===\n");
    printf("\nThis demo used a real image loaded with OpenCV.\n");
    printf("Key technical details:\n");
    printf("• Image loaded in BGR format (OpenCV default)\n");
    printf("• Data passed as row-major order: height × width × channels\n");
    printf("• Face recognition works on actual facial features\n");
    printf("• Multiple thresholds tested to show confidence levels\n");
    
    return 0;
}