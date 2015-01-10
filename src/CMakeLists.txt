# miknet serialization tools
add_library(mikmeta mikmeta.c)

# miknet posix wrapper utility
add_library(miksys miksys.c)

# miknet address storage
add_library(mikaddr mikaddr.c)
target_link_libraries(mikaddr miksys)

# miknet packet handling
add_library(mikpack mikpack.c)
target_link_libraries(mikpack mikmeta)
