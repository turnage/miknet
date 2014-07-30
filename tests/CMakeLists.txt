enable_testing()

# Set up Check and test headers
set(CMAKE_MODULE_PATH "../CMakeModules/")
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})
set(TEST_LIBS ${LIBS} ${CHECK_LIBRARIES})
include_directories(../src/include)

# miklog
add_executable(miklog_test miklog_test.c)
target_link_libraries(miklog_test ${TEST_LIBS} miklog)
add_test(miklog_test ${CMAKE_CURRENT_BINARY_DIR}/miklog_test)

# mikid
add_executable(mikid_test mikid_test.c)
target_link_libraries(mikid_test ${TEST_LIBS} mikid)
add_test(mikid_test ${CMAKE_CURRENT_BINARY_DIR}/mikid_test)
