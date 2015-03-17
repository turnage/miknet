enable_testing()

# Set up Check and test headers
set(CMAKE_MODULE_PATH "../CMakeModules/")
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})
set(TEST_LIBS ${LIBS} ${CHECK_LIBRARIES})

# Include testing utilities
include_directories(util/include)
add_subdirectory(util)

# mikaddr
add_executable(mikaddr_test mikaddr_test.c)
target_link_libraries(mikaddr_test ${TEST_LIBS} mikaddr miksysmock)
add_test(mikaddr_test ${CMAKE_CURRENT_BINARY_DIR}/mikaddr_test)
