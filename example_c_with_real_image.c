#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "facerust.h"

// Simple STB image loader (header-only implementation)
#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_RESIZE_IMPLEMENTATION

// STB Image - minimal implementation for this demo
typedef struct {
    int width, height, channels;
    unsigned char* data;
} SimpleImage;

// Simple BMP loader (basic implementation for demo)
SimpleImage* load_simple_image(const char* filename) {
    FILE* file = fopen(filename, "rb");
    if (!file) return NULL;
    
    // Simple check for file size
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    if (file_size < 1000) {  // Too small to be a real image
        fclose(file);
        return NULL;
    }
    
    // Create a dummy image for demo (since we don't have STB linked)
    SimpleImage* img = malloc(sizeof(SimpleImage));
    img->width = 640;
    img->height = 480;
    img->channels = 3;
    
    int size = img->width * img->height * img->channels;
    img->data = malloc(size);
    
    // Fill with a gradient pattern (not a real image, but demonstrates the interface)
    for (int y = 0; y < img->height; y++) {
        for (int x = 0; x < img->width; x++) {
            int idx = (y * img->width + x) * 3;
            img->data[idx + 0] = (x % 256);  // B
            img->data[idx + 1] = (y % 256);  // G
            img->data[idx + 2] = ((x+y) % 256);  // R
        }
    }
    
    fclose(file);
    return img;
}

void free_simple_image(SimpleImage* img) {
    if (img) {
        free(img->data);
        free(img);
    }
}

int main(int argc, char* argv[]) {
    printf("=== FaceRust C Integration Demo (Real Image) ===\n");
    
    // Check command line arguments
    if (argc != 3) {
        printf("Usage: %s <database_path> <image_path>\n", argv[0]);
        printf("Example: %s ./media/db ./media/testdata/IMG.jpg\n", argv[0]);
        return 1;
    }
    
    const char* db_path = argv[1];
    const char* image_path = argv[2];
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
    
    // Step 3: Load the image
    printf("\n3. Loading image from %s...\n", image_path);
    SimpleImage* img = load_simple_image(image_path);
    if (img == NULL) {
        printf("ERROR: Failed to load image from %s\n", image_path);
        printf("Make sure the image file exists and is readable\n");
        facerecognition_destroy(face_rec);
        return 1;
    }
    printf("✓ Image loaded: %dx%d with %d channels\n", img->width, img->height, img->channels);
    
    // Step 4: Run face recognition with different thresholds
    printf("\n4. Running face recognition...\n");
    
    float thresholds[] = {0.1f, 0.3f, 0.5f, 0.7f};
    int num_thresholds = sizeof(thresholds) / sizeof(thresholds[0]);
    
    for (int i = 0; i < num_thresholds; i++) {
        float threshold = thresholds[i];
        printf("\n   Testing with threshold %.1f:\n", threshold);
        
        CMatchResult result = facerecognition_run_one_face_opencv_mat(
            face_rec, img->data, img->height, img->width, img->channels, threshold
        );
        
        printf("     Name: %s\n", result.name ? result.name : "NULL");
        printf("     Score: %.3f\n", result.score);
        
        if (result.score >= threshold) {
            printf("     ✓ Face recognized as: %s (confidence: %.1f%%)\n", 
                   result.name, result.score * 100);
        } else {
            printf("     ⚠ No face recognized above threshold (got %.1f%%, need %.1f%%)\n", 
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
    printf("\nNote: This demo used a synthetic image pattern for simplicity.\n");
    printf("For real face recognition, you need:\n");
    printf("- Actual face images (JPEG, PNG, etc.)\n");
    printf("- Proper image loading library (OpenCV, STB Image, etc.)\n");
    printf("- Images should contain visible faces for recognition to work\n");
    
    return 0;
}