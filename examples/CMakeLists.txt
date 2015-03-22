# A simple mikgram sender.
add_executable(simple_send simple_send.c)
target_link_libraries(simple_send miknode mikgram)
