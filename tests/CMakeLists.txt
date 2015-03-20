enable_testing()

# Include testing utilities
include_directories(util/include)
add_subdirectory(util)

# mikaddr
add_executable(mikaddr_test mikaddr_test.c)
target_link_libraries(mikaddr_test ${CHECK_LIBRARIES} mikaddr miksysmock)
set_target_properties(mikaddr_test PROPERTIES LINK_FLAGS ${CHECK_CFLAGS})
add_test(mikaddr_test ${CMAKE_CURRENT_BINARY_DIR}/mikaddr_test)

# mikgram
add_executable(mikgram_test mikgram_test.c)
target_link_libraries(mikgram_test ${CHECK_LIBRARIES} mikgram)
set_target_properties(mikgram_test PROPERTIES LINK_FLAGS ${CHECK_CFLAGS})
add_test(mikgram_test ${CMAKE_CURRENT_BINARY_DIR}/mikgram_test)

# miknode
add_executable(miknode_test miknode_test.c)
target_link_libraries(miknode_test ${CHECK_LIBRARIES} miknode miksysmock)
set_target_properties(miknode_test PROPERTIES LINK_FLAGS ${CHECK_CFLAGS})
add_test(miknode_test ${CMAKE_CURRENT_BINARY_DIR}/miknode_test)
