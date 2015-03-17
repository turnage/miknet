# miknet posix wrapper utility
add_library(miksys miksys.c)

# miknet address storage
add_library(mikaddr mikaddr.c)
target_link_libraries(mikaddr miksys)

#miknet node/user interface
add_library(miknode miknode.c)
target_link_libraries(miknode mikaddr miksys)
