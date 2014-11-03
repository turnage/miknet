enable_testing()

# Set up Check and test headers
set(CMAKE_MODULE_PATH "../CMakeModules/")
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})
set(TEST_LIBS ${LIBS} ${CHECK_LIBRARIES})

# Include testing utilities
include_directories(util/include)
add_subdirectory(util)

# mikid
add_executable(mikid_test mikid_test.c)
target_link_libraries(mikid_test ${TEST_LIBS} mikid)
add_test(mikid_test ${CMAKE_CURRENT_BINARY_DIR}/mikid_test)

# mikmeta
add_executable(mikmeta_test mikmeta_test.c)
target_link_libraries(mikmeta_test ${TEST_LIBS} mikmeta)
add_test(mikmeta_test ${CMAKE_CURRENT_BINARY_DIR}/mikmeta_test)
