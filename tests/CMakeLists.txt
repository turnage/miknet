enable_testing()

# Include testing utilities
include_directories(util/include)
add_subdirectory(util)

# mikaddr
add_executable(mikaddr_test mikaddr_test.c)
target_link_libraries(mikaddr_test ${CHECK_LIBRARIES} mikaddr miksysmock)
set_target_properties(mikaddr_test PROPERTIES LINK_FLAGS ${CHECK_CFLAGS})
add_test(mikaddr_test ${CMAKE_CURRENT_BINARY_DIR}/mikaddr_test)
