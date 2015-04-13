cmake_minimum_required(VERSION 2.6)
project(miknet)

# Release build.
include_directories(include)
add_subdirectory(src)
add_subdirectory(examples)

# Testing/Debug build.
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
	set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -Wall -Werror -pedantic")
	set(CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
	add_subdirectory(tests)
endif()
