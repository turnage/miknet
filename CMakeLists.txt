cmake_minimum_required(VERSION 2.6)
project(miknet)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
	set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -Wall -Werror -pedantic")
	enable_testing()
	add_subdirectory(tests)
endif()

add_subdirectory(src)

