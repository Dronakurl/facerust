use opencv::{core::Mat, core::Rect2i, core::Size, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbLoadStatus {
    NotLoaded,
    Loading,
    Loaded,
}

impl std::fmt::Display for DbLoadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbLoadStatus::NotLoaded => write!(f, "NOT_LOADED"),
            DbLoadStatus::Loading => write!(f, "LOADING"),
            DbLoadStatus::Loaded => write!(f, "LOADED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub name: String,
    pub score: f32,
}

impl MatchResult {
    pub fn new(name: String, score: f32) -> Self {
        Self { name, score }
    }

    pub fn to_lower_case(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn is_unknown(&self) -> bool {
        self.to_lower_case() == "unknown"
    }

    pub fn to_string(&self) -> String {
        if self.is_unknown() {
            self.name.clone()
        } else {
            format!("{} ({:.2})", self.name, self.score)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchResults {
    pub results: Vec<MatchResult>,
    pub best_match: MatchResult,
}

#[derive(Debug, Clone)]
pub struct DetectedFace {
    pub name: String,
    pub face_detect: Mat,
    pub feature: Mat,
    pub original_size: Size,
}

impl DetectedFace {
    pub fn new(name: String, face_detect: Mat, feature: Mat, original_size: Size) -> Self {
        Self {
            name,
            face_detect,
            feature,
            original_size,
        }
    }

    pub fn bbox(&self) -> opencv::Result<Rect2i> {
        if self.face_detect.empty() {
            return Ok(Rect2i::default());
        }

        let x = *self.face_detect.at_2d::<f32>(0, 0)? as i32;
        let y = *self.face_detect.at_2d::<f32>(0, 1)? as i32;
        let w = *self.face_detect.at_2d::<f32>(0, 2)? as i32;
        let h = *self.face_detect.at_2d::<f32>(0, 3)? as i32;

        Ok(Rect2i::new(x, y, w, h))
    }
}