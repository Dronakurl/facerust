# 🧠 ONNX Models for FaceRust

FaceRust uses two OpenCV deep learning models for face detection and recognition:

## 📦 Models Used

| Model | Purpose | Source | Size |
|-------|---------|--------|------|
| **YuNet 2023 Mar** | Face Detection | OpenCV Zoo | ~340KB |
| **SFace 2021 Dec** | Face Recognition | OpenCV Zoo | ~37MB |

## 🔄 Automatic Download

Models are **automatically downloaded** during `cargo build`:

```bash
cargo build --release
# ✅ Downloads models if missing
# ⚡ Skips if already present
```

**Build output:**
```
warning: facerust@0.1.0: ✓ Successfully downloaded: models/face_detection_yunet_2023mar.onnx
warning: facerust@0.1.0: ✓ Successfully downloaded: models/face_recognition_sface_2021dec.onnx
```

## 📁 Model Locations

```
facerust/
└── models/
    ├── face_detection_yunet_2023mar.onnx    # Face detection
    └── face_recognition_sface_2021dec.onnx  # Face recognition
```

## 🛠️ Manual Download

If automatic download fails, use the provided script:

```bash
./download_models.sh
```

**Or download manually:**
```bash
# Face Detection Model
curl -L -o models/face_detection_yunet_2023mar.onnx \
  'https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx'

# Face Recognition Model  
curl -L -o models/face_recognition_sface_2021dec.onnx \
  'https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx'
```

## 🔧 How It Works

1. **Build Script** (`build.rs`) runs during compilation
2. **Checks** if models exist in `models/` directory
3. **Downloads** missing models using `curl` or `wget`
4. **Continues** build if download succeeds
5. **Warns** if download fails (build continues)

## ⚙️ Requirements

- **curl** or **wget** (for automatic download)
- **Internet connection** (for first build)
- **~38MB** disk space for models

## 🐛 Troubleshooting

**"Failed to download models"**
- Check internet connection
- Install `curl` or `wget`
- Run manual download script: `./download_models.sh`

**"Model files not found"**
- Ensure models are in `models/` directory
- Verify file names match exactly
- Re-run `./download_models.sh`

**"Layer with requested id=-1 not found"**
- Your OpenCV version is too old
- Run `./install_opencv.sh` to get OpenCV 4.11.0+

---

The model system is designed to be **zero-configuration** - just run `cargo build` and everything works! 🚀