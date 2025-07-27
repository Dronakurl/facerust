# Simple Makefile for FaceRust C Integration Demo
CC = gcc
CFLAGS = -Wall -Wextra -std=c99 -O2
LDFLAGS = -L./target/release -lfacerust -lpthread -ldl -lm

# Program name
PROGRAM = face_demo

# Source files
SOURCES = example_c_integration.c

# Default target
all: $(PROGRAM)

# Build the demo program
$(PROGRAM): $(SOURCES) target/release/libfacerust.so
	$(CC) $(CFLAGS) -o $(PROGRAM) $(SOURCES) $(LDFLAGS)
	@echo "✓ Built $(PROGRAM) successfully"
	@echo ""
	@echo "Usage: ./$(PROGRAM) <database_path>"
	@echo "Example: ./$(PROGRAM) ./media/db"

# Build the Rust library if needed
target/release/libfacerust.so:
	@echo "Building Rust library..."
	cargo build --release

# Test with the demo database
test: $(PROGRAM)
	@echo "Running demo with media/db..."
	LD_LIBRARY_PATH=./target/release ./$(PROGRAM) ./media/db

# Clean build artifacts
clean:
	rm -f $(PROGRAM)
	cargo clean
	@echo "✓ Cleaned build artifacts"

# Install system dependencies (Ubuntu/Debian)
install-deps:
	@echo "Installing system dependencies..."
	sudo apt-get update
	sudo apt-get install -y build-essential pkg-config libopencv-dev

# Download ONNX models
download-models:
	@echo "Downloading ONNX models..."
	./download_models.sh

# Help
help:
	@echo "FaceRust C Integration Demo"
	@echo ""
	@echo "Available targets:"
	@echo "  all            - Build the demo program (default)"
	@echo "  test           - Run the demo with sample database"
	@echo "  download-models - Download ONNX models manually"
	@echo "  clean          - Clean build artifacts"
	@echo "  install-deps   - Install system dependencies"
	@echo "  help           - Show this help"
	@echo ""
	@echo "Usage:"
	@echo "  make                 # Build the demo"
	@echo "  make test            # Run the demo"
	@echo "  make download-models # Download models manually"
	@echo "  make clean           # Clean up"

.PHONY: all test clean install-deps download-models help