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

# mikmemapi
add_executable(mikmemapi_test mikmemapi_test.c)
target_link_libraries(mikmemapi_test ${TEST_LIBS} mikmemapi)
add_test(mikmemapi_test ${CMAKE_CURRENT_BINARY_DIR}/mikmemapi_test)

# mikmemmock
add_executable(mikmemmock_test mikmemmock_test.c)
target_link_libraries(mikmemmock_test ${TEST_LIBS} mikmemmock)
add_test(mikmemmock_test ${CMAKE_CURRENT_BINARY_DIR}/mikmemmock_test)
