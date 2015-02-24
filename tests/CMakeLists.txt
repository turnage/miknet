enable_testing()

# Set up Check and test headers
set(CMAKE_MODULE_PATH "../CMakeModules/")
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})
set(TEST_LIBS ${LIBS} ${CHECK_LIBRARIES})

# Include testing utilities
include_directories(util/include)
add_subdirectory(util)

# mikmeta
add_executable(mikmeta_test mikmeta_test.c)
target_link_libraries(mikmeta_test ${TEST_LIBS} mikmeta)
add_test(mikmeta_test ${CMAKE_CURRENT_BINARY_DIR}/mikmeta_test)

# mikaddr
add_executable(mikaddr_test mikaddr_test.c)
target_link_libraries(mikaddr_test ${TEST_LIBS} mikaddr miksysmock)
add_test(mikaddr_test ${CMAKE_CURRENT_BINARY_DIR}/mikaddr_test)

# mikpack
add_executable(mikpack_test mikpack_test.c)
target_link_libraries(mikpack_test ${TEST_LIBS} mikpack mikmeta)
add_test(mikpack_test ${CMAKE_CURRENT_BINARY_DIR}/mikpack_test)

# miklist
add_executable(miklist_test miklist_test.c)
target_link_libraries(miklist_test ${TEST_LIBS} miklist)
add_test(miklist_test ${CMAKE_CURRENT_BINARY_DIR}/miklist_test)
