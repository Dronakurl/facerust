#!/bin/bash

# OpenCV 4.11.0 Installation Script for FaceRust
# This script installs OpenCV 4.11.0 to fix compatibility with YuNet ONNX models

set -e

echo "🦀 FaceRust OpenCV 4.11.0 Installation Script"
echo "=============================================="

# Check if script is run as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ Please don't run this script as root. Use regular user with sudo access."
   exit 1
fi

# Install dependencies
echo "📦 Installing build dependencies..."
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
    build-essential cmake git pkg-config \
    libjpeg-dev libpng-dev libtiff-dev \
    libavcodec-dev libavformat-dev libswscale-dev \
    libgtk2.0-dev libcanberra-gtk-module \
    python3-dev python3-numpy \
    libtbb2 libtbb-dev libdc1394-22-dev \
    libv4l-dev libgtkglext1-dev \
    libatlas-base-dev gfortran \
    libhdf5-dev libprotobuf-dev \
    libgoogle-glog-dev libgflags-dev \
    libgphoto2-dev libeigen3-dev libhdf5-dev

# Create build directory
OPENCV_BUILD_DIR="$HOME/opencv_build"
echo "🏗️  Creating build directory: $OPENCV_BUILD_DIR"
mkdir -p "$OPENCV_BUILD_DIR"
cd "$OPENCV_BUILD_DIR"

# Clone OpenCV repositories
echo "📥 Downloading OpenCV 4.11.0..."
if [ ! -d "opencv" ]; then
    git clone https://github.com/opencv/opencv.git
fi
cd opencv
git checkout 4.11.0
cd ..

if [ ! -d "opencv_contrib" ]; then
    git clone https://github.com/opencv/opencv_contrib.git
fi
cd opencv_contrib
git checkout 4.11.0
cd ..

# Create build directory
echo "🔨 Configuring OpenCV build..."
mkdir -p build
cd build

# Configure with CMake (without CUDA for local development)
cmake -D CMAKE_BUILD_TYPE=Release \
    -D CMAKE_INSTALL_PREFIX=/usr/local \
    -D OPENCV_EXTRA_MODULES_PATH=../opencv_contrib/modules \
    -D BUILD_opencv_python3=ON \
    -D OPENCV_ENABLE_NONFREE=ON \
    -D BUILD_EXAMPLES=OFF \
    -D BUILD_TESTS=OFF \
    -D BUILD_PERF_TESTS=OFF \
    -D WITH_V4L=ON \
    -D WITH_QT=OFF \
    -D WITH_OPENGL=ON \
    -D WITH_TBB=ON \
    ../opencv

# Build OpenCV
echo "🚀 Building OpenCV (this may take 15-30 minutes)..."
NPROC=$(nproc)
echo "Using $NPROC CPU cores for compilation..."
make -j"$NPROC"

# Install OpenCV
echo "📦 Installing OpenCV..."
sudo make install
sudo ldconfig

# Verify installation
echo "✅ Verifying OpenCV installation..."
OPENCV_VERSION=$(opencv_version 2>/dev/null || echo "Not found in PATH")
echo "OpenCV version: $OPENCV_VERSION"

if [ "$OPENCV_VERSION" = "4.11.0" ]; then
    echo "🎉 OpenCV 4.11.0 successfully installed!"
    echo ""
    echo "📋 Next steps:"
    echo "1. Restart your terminal or run: source ~/.bashrc"
    echo "2. Go to facerust directory: cd /path/to/facerust"
    echo "3. Clean and rebuild: cargo clean && cargo build"
    echo "4. Test face recognition: cargo run --bin facerust-cli"
    echo ""
    echo "🗑️  To clean up build files (optional):"
    echo "   rm -rf $OPENCV_BUILD_DIR"
else
    echo "⚠️  OpenCV installation may have issues. Version check returned: $OPENCV_VERSION"
    echo "You may need to restart your terminal and check your PATH."
fi

echo ""
echo "🦀 OpenCV installation complete!"