   set(CMAKE_SYSTEM_NAME Windows)
   set(CMAKE_SYSTEM_VERSION 1)
   
   # Specify the cross compiler
   set(CMAKE_C_COMPILER x86_64-w64-mingw32-gcc)
   set(CMAKE_CXX_COMPILER x86_64-w64-mingw32-g++)
   
   # Specify the target environment
   set(CMAKE_C_FLAGS "-static")
   set(CMAKE_CXX_FLAGS "-static")
   
   # Adjust the paths if necessary
   # set(CMAKE_FIND_ROOT_PATH /usr/x86_64-w64-mingw32)
   
   # Search for programs in the build host directories
   set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
   
   # Adjust the library paths
   set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
   set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)