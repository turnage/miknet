enable_testing()

# Include testing utilities
find_package(Check REQUIRED)
include_directories(${CHECK_INCLUDE_DIRS})

find_package(Threads REQUIRED)

include_directories(util/include)
add_subdirectory(util)

# mikaddr
add_executable(mikaddr_test mikaddr_test.c)
target_link_libraries(mikaddr_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					mikaddr miksysmock)
add_test(mikaddr_test ${CMAKE_CURRENT_BINARY_DIR}/mikaddr_test)

# miktime
add_executable(miktime_test miktime_test.c)
target_link_libraries(miktime_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					miktime)
add_test(miktime_test ${CMAKE_CURRENT_BINARY_DIR}/miktime_test)

# mikgram
add_executable(mikgram_test mikgram_test.c)
target_link_libraries(mikgram_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					mikgram)
add_test(mikgram_test ${CMAKE_CURRENT_BINARY_DIR}/mikgram_test)

# mikmsg
add_executable(mikmsg_test mikmsg_test.c)
target_link_libraries(mikmsg_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					mikmsg mikgram)
add_test(mikmsg_test ${CMAKE_CURRENT_BINARY_DIR}/mikmsg_test)

# mikstation
add_executable(mikstation_test mikstation_test.c)
target_link_libraries(mikstation_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					mikstation miksysmock)
add_test(mikstation_test ${CMAKE_CURRENT_BINARY_DIR}/mikstation_test)

# miknode
add_executable(miknode_test miknode_test.c)
target_link_libraries(miknode_test	${CHECK_LIBRARIES}
					${CMAKE_THREAD_LIBS_INIT}
					miknode miksysmock)
add_test(miknode_test ${CMAKE_CURRENT_BINARY_DIR}/miknode_test)
