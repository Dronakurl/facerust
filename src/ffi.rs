use crate::{FaceRecognition, MatchResult};
use opencv::core::Mat;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};
use std::ptr;
use tokio::runtime::Runtime;

// Opaque pointer type for FaceRecognition
pub struct CFaceRecognition {
    inner: FaceRecognition,
    runtime: Runtime,
}

// Match result structure for C
#[repr(C)]
pub struct CMatchResult {
    name: *mut c_char,
    score: c_float,
}

impl From<MatchResult> for CMatchResult {
    fn from(result: MatchResult) -> Self {
        let name_cstring =
            CString::new(result.name).unwrap_or_else(|_| CString::new("error").unwrap());
        Self {
            name: name_cstring.into_raw(),
            score: result.score,
        }
    }
}

#[no_mangle]
pub extern "C" fn facerecognition_create() -> *mut CFaceRecognition {
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let face_rec = match FaceRecognition::new(
        Some("models/face_detection_yunet_2023mar.onnx"),
        Some("models/face_recognition_sface_2021dec.onnx"),
        Some(1000),
    ) {
        Ok(fr) => fr,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CFaceRecognition {
        inner: face_rec,
        runtime,
    }))
}

#[no_mangle]
pub extern "C" fn facerecognition_load_persons_db(
    face_rec: *mut CFaceRecognition,
    db_path: *const c_char,
) -> c_int {
    if face_rec.is_null() || db_path.is_null() {
        return -1;
    }

    let face_rec = unsafe { &mut *face_rec };
    let db_path_str = match unsafe { CStr::from_ptr(db_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match face_rec.runtime.block_on(async {
        face_rec
            .inner
            .load_persons_db(db_path_str, false, false)
            .await
    }) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn facerecognition_run_one_face_opencv_mat(
    face_rec: *mut CFaceRecognition,
    mat_data: *const u8,
    rows: c_int,
    cols: c_int,
    channels: c_int,
    threshold: c_float,
) -> CMatchResult {
    if face_rec.is_null() || mat_data.is_null() {
        return CMatchResult {
            name: CString::new("error").unwrap().into_raw(),
            score: 0.0,
        };
    }

    let face_rec = unsafe { &mut *face_rec };

    // Create OpenCV Mat from raw data
    let mat_type = match channels {
        1 => opencv::core::CV_8UC1,
        3 => opencv::core::CV_8UC3,
        _ => {
            return CMatchResult {
                name: CString::new("error").unwrap().into_raw(),
                score: 0.0,
            }
        }
    };

    let mat = unsafe {
        match Mat::new_rows_cols_with_data_unsafe(
            rows,
            cols,
            mat_type,
            mat_data as *mut _,
            opencv::core::Mat_AUTO_STEP,
        ) {
            Ok(m) => m,
            Err(_) => {
                return CMatchResult {
                    name: CString::new("error").unwrap().into_raw(),
                    score: 0.0,
                }
            }
        }
    };

    let result = face_rec
        .runtime
        .block_on(async { face_rec.inner.run_one_face(mat, threshold, false).await });

    match result {
        Ok(match_result) => match_result.into(),
        Err(_) => CMatchResult {
            name: CString::new("unknown").unwrap().into_raw(),
            score: 0.0,
        },
    }
}

#[no_mangle]
pub extern "C" fn facerecognition_free_match_result(result: *mut CMatchResult) {
    if !result.is_null() {
        unsafe {
            let result = &mut *result;
            if !result.name.is_null() {
                let _ = CString::from_raw(result.name);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn facerecognition_destroy(face_rec: *mut CFaceRecognition) {
    if !face_rec.is_null() {
        unsafe {
            let _ = Box::from_raw(face_rec);
        }
    }
}
