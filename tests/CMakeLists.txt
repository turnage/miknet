enable_testing()

# Set up Check and test headers
set(CMAKE_MODULE_PATH "../CMakeModules/")
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})
set(TEST_LIBS ${LIBS} ${CHECK_LIBRARIES})
include_directories(../src/include)

# miklogger
add_executable(miklogger_test miklogger_test.c)
target_link_libraries(miklogger_test ${TEST_LIBS} miklogger)
add_test(miklogger_test ${CMAKE_CURRENT_BINARY_DIR}/miklogger_test)

