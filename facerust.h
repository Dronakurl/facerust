#ifndef FACERUST_H
#define FACERUST_H

#ifdef __cplusplus
extern "C" {
#endif

// Opaque pointer to the Rust FaceRecognition struct
typedef struct CFaceRecognition CFaceRecognition;

// Match result structure
typedef struct {
    char* name;
    float score;
} CMatchResult;

// Create a new FaceRecognition instance
CFaceRecognition* facerecognition_create();

// Load persons database from directory
int facerecognition_load_persons_db(CFaceRecognition* face_rec, const char* db_path);

// Run face recognition on OpenCV Mat data
CMatchResult facerecognition_run_one_face_opencv_mat(
    CFaceRecognition* face_rec,
    const unsigned char* mat_data,
    int rows,
    int cols, 
    int channels,
    float threshold
);

// Free memory allocated for match result
void facerecognition_free_match_result(CMatchResult* result);

// Destroy FaceRecognition instance
void facerecognition_destroy(CFaceRecognition* face_rec);

#ifdef __cplusplus
}
#endif

#endif // FACERUST_H