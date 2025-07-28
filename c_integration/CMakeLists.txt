cmake_minimum_required(VERSION 3.16)
project(FaceRustCDemo)

# Export compile commands for language servers (clangd, etc.)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Set C and CXX standards (we need C++ for OpenCV headers)
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Set default build type to Release
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Add compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra")

# Find required packages
find_package(PkgConfig REQUIRED)

# Try to find OpenCV (required for static linking)
find_package(OpenCV QUIET)
if(OpenCV_FOUND)
    message(STATUS "Found OpenCV ${OpenCV_VERSION}")
    set(OPENCV_AVAILABLE TRUE)
else()
    # Try pkg-config as fallback
    pkg_check_modules(OPENCV opencv4)
    if(NOT OPENCV_FOUND)
        pkg_check_modules(OPENCV opencv)
    endif()
    
    if(OPENCV_FOUND)
        message(STATUS "Found OpenCV via pkg-config")
        set(OPENCV_AVAILABLE TRUE)
    else()
        message(WARNING "OpenCV not found - static linking will not work")
        set(OPENCV_AVAILABLE FALSE)
    endif()
endif()

# Set paths
set(FACERUST_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/..")
set(FACERUST_TARGET_DIR "${FACERUST_ROOT_DIR}/target/release")
set(FACERUST_STATIC_LIB "${FACERUST_TARGET_DIR}/libfacerust.a")
set(FACERUST_DYNAMIC_LIB "${FACERUST_TARGET_DIR}/libfacerust.so")

# Custom target to build Rust library
add_custom_target(build_rust_lib
    COMMAND cargo build --release
    WORKING_DIRECTORY "${FACERUST_ROOT_DIR}"
    COMMENT "Building Rust facerust library"
)

# Check if libraries exist and create targets accordingly
if(EXISTS "${FACERUST_STATIC_LIB}" OR EXISTS "${FACERUST_DYNAMIC_LIB}")
    message(STATUS "Rust libraries found, creating targets")
else()
    message(STATUS "Rust libraries not found, will build them first")
endif()

# Source files
set(SOURCES example_c.cpp)

# Option for linking type
option(USE_STATIC_LINKING "Use static linking instead of dynamic" OFF)

if(USE_STATIC_LINKING AND OPENCV_AVAILABLE)
    # Static linking version
    add_executable(face_demo_static ${SOURCES})
    add_dependencies(face_demo_static build_rust_lib)
    
    # Link static library
    target_link_libraries(face_demo_static 
        "${FACERUST_STATIC_LIB}"
        pthread
        dl
        m
        stdc++
    )
    
    # Add OpenCV libraries for static linking
    if(OpenCV_FOUND)
        target_link_libraries(face_demo_static ${OpenCV_LIBS})
        target_include_directories(face_demo_static PRIVATE ${OpenCV_INCLUDE_DIRS})
    else()
        target_link_libraries(face_demo_static ${OPENCV_LIBRARIES})
        target_include_directories(face_demo_static PRIVATE ${OPENCV_INCLUDE_DIRS})
        target_compile_options(face_demo_static PRIVATE ${OPENCV_CFLAGS_OTHER})
    endif()
    
    message(STATUS "Building static version: face_demo_static")
    
else()
    # Dynamic linking version (default)
    add_executable(face_demo ${SOURCES})
    add_dependencies(face_demo build_rust_lib)
    
    # Link dynamic library
    target_link_libraries(face_demo 
        pthread
        dl
        m
        stdc++ 
    )
    
    # Add library path and link facerust
    target_link_directories(face_demo PRIVATE "${FACERUST_TARGET_DIR}")
    target_link_libraries(face_demo facerust)
    
    # Add OpenCV for image loading
    if(OpenCV_FOUND)
        target_link_libraries(face_demo ${OpenCV_LIBS})
        target_include_directories(face_demo PRIVATE ${OpenCV_INCLUDE_DIRS})
    else()
        target_link_libraries(face_demo ${OPENCV_LIBRARIES})
        target_include_directories(face_demo PRIVATE ${OPENCV_INCLUDE_DIRS})
        target_compile_options(face_demo PRIVATE ${OPENCV_CFLAGS_OTHER})
    endif()
    
    message(STATUS "Building dynamic version: face_demo")
    message(STATUS "Remember to set LD_LIBRARY_PATH=${FACERUST_TARGET_DIR} when running")
endif()

# Custom target for testing
if(USE_STATIC_LINKING AND OPENCV_AVAILABLE)
    add_custom_target(test_demo
        COMMAND ./c_integration/build/face_demo_static media/db
        DEPENDS face_demo_static
        WORKING_DIRECTORY "${FACERUST_ROOT_DIR}"
        COMMENT "Running face_demo_static with test database"
    )
else()
    add_custom_target(test_demo
        COMMAND ${CMAKE_COMMAND} -E env LD_LIBRARY_PATH=${FACERUST_TARGET_DIR} ./c_integration/build/face_demo media/db
        DEPENDS face_demo
        WORKING_DIRECTORY "${FACERUST_ROOT_DIR}"
        COMMENT "Running face_demo with test database"
    )
endif()

# Print configuration summary
message(STATUS "")
message(STATUS "Configuration Summary:")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  Static Linking: ${USE_STATIC_LINKING}")
message(STATUS "  OpenCV Available: ${OPENCV_AVAILABLE}")
message(STATUS "  Rust Library Path: ${FACERUST_TARGET_DIR}")
message(STATUS "")
message(STATUS "Usage:")
if(USE_STATIC_LINKING AND OPENCV_AVAILABLE)
    message(STATUS "  cd .. && ./c_integration/build/face_demo_static media/db")
else()
    message(STATUS "  cd .. && LD_LIBRARY_PATH=${FACERUST_TARGET_DIR} ./c_integration/build/face_demo media/db")
endif()
message(STATUS "  or: cmake --build . --target test_demo (runs from correct directory automatically)")
message(STATUS "")