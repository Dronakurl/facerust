#pragma once

#include "facerust.h"
#include <opencv2/opencv.hpp>
#include <string>
#include <memory>

// Match result compatible with your existing code
struct MatchResult {
    std::string name;
    float score;
    
    MatchResult(const std::string& n = "Unknown", float s = 0.0f) 
        : name(n), score(s) {}
    
    bool isUnknown() const {
        std::string lower_name = name;
        std::transform(lower_name.begin(), lower_name.end(), lower_name.begin(), ::tolower);
        return lower_name == "unknown";
    }
    
    std::string toString() const {
        if (isUnknown()) return name;
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(2) << score;
        return name + " (" + oss.str() + ")";
    }
};

// RAII wrapper for the Rust FaceRecognition
class FaceRecognitionRust {
private:
    CFaceRecognition* face_rec_;

public:
    FaceRecognitionRust() {
        face_rec_ = facerecognition_create();
        if (!face_rec_) {
            throw std::runtime_error("Failed to create FaceRecognition instance");
        }
    }
    
    ~FaceRecognitionRust() {
        if (face_rec_) {
            facerecognition_destroy(face_rec_);
        }
    }
    
    // Delete copy constructor and assignment operator
    FaceRecognitionRust(const FaceRecognitionRust&) = delete;
    FaceRecognitionRust& operator=(const FaceRecognitionRust&) = delete;
    
    // Move constructor and assignment operator
    FaceRecognitionRust(FaceRecognitionRust&& other) noexcept : face_rec_(other.face_rec_) {
        other.face_rec_ = nullptr;
    }
    
    FaceRecognitionRust& operator=(FaceRecognitionRust&& other) noexcept {
        if (this != &other) {
            if (face_rec_) {
                facerecognition_destroy(face_rec_);
            }
            face_rec_ = other.face_rec_;
            other.face_rec_ = nullptr;
        }
        return *this;
    }
    
    void loadPersonsDB(const std::string& db_path) {
        if (facerecognition_load_persons_db(face_rec_, db_path.c_str()) != 0) {
            throw std::runtime_error("Failed to load persons database: " + db_path);
        }
    }
    
    MatchResult run_one_face(const cv::Mat& image, float threshold = 0.3f) {
        if (image.empty()) {
            return MatchResult("Unknown", 0.0f);
        }
        
        // Ensure the image is in BGR format
        cv::Mat bgr_image;
        if (image.channels() == 3) {
            bgr_image = image;
        } else if (image.channels() == 1) {
            cv::cvtColor(image, bgr_image, cv::COLOR_GRAY2BGR);
        } else {
            return MatchResult("Unknown", 0.0f);
        }
        
        // Make sure the image is continuous in memory
        if (!bgr_image.isContinuous()) {
            bgr_image = bgr_image.clone();
        }
        
        CMatchResult c_result = facerecognition_run_one_face_opencv_mat(
            face_rec_,
            bgr_image.data,
            bgr_image.rows,
            bgr_image.cols,
            bgr_image.channels(),
            threshold
        );
        
        MatchResult result;
        if (c_result.name) {
            result.name = std::string(c_result.name);
            result.score = c_result.score;
            facerecognition_free_match_result(&c_result);
        } else {
            result = MatchResult("Unknown", 0.0f);
        }
        
        return result;
    }
};