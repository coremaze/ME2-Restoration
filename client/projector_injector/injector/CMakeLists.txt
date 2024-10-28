cmake_minimum_required(VERSION 3.10)

# Set the project name and specify the C++ standard
project(injector VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Add the executable
add_executable(injector main.cpp)

# Specify Windows-specific libraries if needed
if(WIN32)
    target_link_libraries(injector PRIVATE)
endif()
