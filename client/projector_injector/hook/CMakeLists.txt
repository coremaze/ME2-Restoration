cmake_minimum_required(VERSION 3.10)

# Set the project name and specify the C++ standard
project(HookLibrary VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Define the shared library (DLL)
add_library(HookLibrary SHARED hook.cpp)

# Link against necessary Windows libraries
target_link_libraries(HookLibrary PRIVATE ws2_32)

# Specify the output directory
set_target_properties(HookLibrary PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)
