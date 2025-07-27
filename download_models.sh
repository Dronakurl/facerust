#!/bin/bash

# FaceRust ONNX Model Downloader
# Downloads the required OpenCV DNN models for face detection and recognition
# It downloads the model in the directory named "models" relative to the script location

set -e # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 FaceRust Model Downloader${NC}"
echo "Downloading ONNX models for face detection and recognition..."
echo

# Create models directory
MODELS_DIR="models"
if [ ! -d "$MODELS_DIR" ]; then
  echo -e "${YELLOW}📁 Creating models directory...${NC}"
  mkdir -p "$MODELS_DIR"
fi

# Model URLs and filenames
declare -A MODELS
MODELS["face_detection_yunet_2023mar.onnx"]="https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx"
MODELS["face_recognition_sface_2021dec.onnx"]="https://github.com/opencv/opencv_zoo/raw/refs/heads/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx"

# Function to download a file
download_model() {
  local filename="$1"
  local url="$2"
  local filepath="$MODELS_DIR/$filename"

  if [ -f "$filepath" ]; then
    echo -e "${GREEN}✓ Already exists: $filename${NC}"
    return 0
  fi

  echo -e "${YELLOW}⬇️  Downloading: $filename${NC}"
  echo "   URL: $url"

  # Try curl first, then wget
  if command -v curl >/dev/null 2>&1; then
    if curl -L -f -s -o "$filepath" "$url"; then
      echo -e "${GREEN}✓ Downloaded: $filename${NC}"
    else
      echo -e "${RED}✗ Failed to download: $filename${NC}"
      return 1
    fi
  elif command -v wget >/dev/null 2>&1; then
    if wget -q -O "$filepath" "$url"; then
      echo -e "${GREEN}✓ Downloaded: $filename${NC}"
    else
      echo -e "${RED}✗ Failed to download: $filename${NC}"
      return 1
    fi
  else
    echo -e "${RED}✗ Neither curl nor wget found. Please install one of them.${NC}"
    echo "   Or download manually from: $url"
    return 1
  fi
}

# Download all models
failed_downloads=0
for filename in "${!MODELS[@]}"; do
  url="${MODELS[$filename]}"
  if ! download_model "$filename" "$url"; then
    ((failed_downloads++))
  fi
  echo
done

# Summary
echo -e "${BLUE}📊 Download Summary:${NC}"
if [ $failed_downloads -eq 0 ]; then
  echo -e "${GREEN}✅ All models downloaded successfully!${NC}"
  echo
  echo -e "${BLUE}📂 Models location:${NC}"
  echo "   $(pwd)/$MODELS_DIR/"
  echo
  echo -e "${BLUE}🚀 You can now build and run FaceRust:${NC}"
  echo "   cargo build --release"
  echo "   cargo run --bin facerust-cli -- -i image.jpg -d ./media/db"
else
  echo -e "${RED}⚠️  $failed_downloads model(s) failed to download${NC}"
  echo
  echo -e "${YELLOW}📋 Manual download instructions:${NC}"
  echo "If automatic download failed, you can download the models manually:"
  echo
  for filename in "${!MODELS[@]}"; do
    url="${MODELS[$filename]}"
    filepath="$MODELS_DIR/$filename"
    if [ ! -f "$filepath" ]; then
      echo "curl -L -o $filepath '$url'"
    fi
  done
  echo
  exit 1
fi

echo -e "${GREEN}🎉 Ready to use FaceRust!${NC}"

