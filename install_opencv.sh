#!/bin/bash

set -e

echo "🦀 FaceRust OpenCV 4.12.0 Installation Script"
echo "=============================================="

# Check if script is run as root
if [[ $EUID -eq 0 ]]; then
   echo "❌ Please don't run this script as root. Use regular user with sudo access."
   exit 1
fi

INSTALL_DIR="$HOME/opencv"

mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR" || exit

if [ -z "$(ls -A $INSTALL_DIR)" ]; then
  git clone https://github.com/opencv/opencv &&
    git -C opencv checkout 4.12.0 &&
    git clone https://github.com/opencv/opencv_contrib &&
    git -C opencv_contrib checkout 4.12.0 &&
    git clone https://github.com/opencv/opencv_extra &&
    git -C opencv_extra checkout 4.12.0 &&
    mkdir -p build
fi

mkdir -p $INSTALL_DIR/build
cd $INSTALL_DIR/build || exit
cmake -D CMAKE_BUILD_TYPE=Release \
  -D CMAKE_INSTALL_PREFIX=/usr/local \
  -D WITH_CUDA=off \
  -D ENABLE_FAST_MATH=1 \
  -D CUDA_FAST_MATH=0 \
  -D WITH_CUBLAS=0 \
  -D OPENCV_DNN_CUDA=OFF \
  -D OPENCV_EXTRA_MODULES_PATH=../opencv_contrib/modules \
  -D BUILD_opencv_cudacodec=OFF \
  ../opencv &&
  make -j"$(nproc)" &&
  make install
