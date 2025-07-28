#!/bin/bash

set -e

echo "=== FaceRust C Integration Build Script ==="
echo ""

# Parse command line arguments
CLEAN=false
STATIC=false
TEST=false
USE_BEAR=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --static)
            STATIC=true
            shift
            ;;
        --test)
            TEST=true
            shift
            ;;
        --bear)
            USE_BEAR=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --clean    Clean build directory first"
            echo "  --static   Use static linking (requires OpenCV)"
            echo "  --test     Run demo after building"
            echo "  --bear     Use bear to generate compile_commands.json for language servers"
            echo "  --help     Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                # Build dynamic version"
            echo "  $0 --static       # Build static version"
            echo "  $0 --clean --test # Clean build and test"
            echo "  $0 --bear         # Generate compile_commands.json with bear"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo "🧹 Cleaning build directory..."
    rm -rf build/
fi

# Create build directory
mkdir -p build
cd build

# Run CMake
echo "⚙️  Configuring CMake..."
CMAKE_ARGS=""
if [ "$STATIC" = true ]; then
    CMAKE_ARGS="-DUSE_STATIC_LINKING=ON"
    echo "Using static linking (requires OpenCV)"
else
    echo "Using dynamic linking"
fi

cmake .. $CMAKE_ARGS

# Build
echo ""
if [ "$USE_BEAR" = true ]; then
    # Check if bear is available
    if ! command -v bear &> /dev/null; then
        echo "❌ Error: bear is not installed!"
        echo "Install with: sudo apt-get install bear"
        exit 1
    fi
    
    echo "🔨 Building with bear (generating compile_commands.json)..."
    bear -- cmake --build . -j$(nproc)
    
    # Copy compile_commands.json to parent directory for language server
    if [ -f "compile_commands.json" ]; then
        cp compile_commands.json ../
        echo "📄 Generated compile_commands.json for language server"
    fi
else
    echo "🔨 Building..."
    cmake --build . -j$(nproc)
    
    # Copy CMake-generated compile_commands.json to parent directory
    if [ -f "compile_commands.json" ]; then
        cp compile_commands.json ../
        echo "📄 Generated compile_commands.json via CMake for language server"
    fi
fi

echo ""
echo "✅ Build completed successfully!"

# Test if requested
if [ "$TEST" = true ]; then
    echo ""
    echo "🧪 Running test..."
    cmake --build . --target test_demo
fi

echo ""
echo "🎉 Done!"
if [ "$STATIC" = true ]; then
    echo "Run: ./build/face_demo_static ../media/db"
else
    echo "Run: LD_LIBRARY_PATH=../target/release ./build/face_demo ../media/db"
fi

if [ "$USE_BEAR" = true ]; then
    echo ""
    echo "📄 compile_commands.json generated for language server support"
    echo "Your language server should now have full IntelliSense support"
fi