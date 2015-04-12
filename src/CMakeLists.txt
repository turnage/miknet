# miknet posix wrapper utility
add_library(miksys miksys.c)

# miknet time utility
add_library(miktime miktime.c)

# miknet address storage
add_library(mikaddr mikaddr.c)
target_link_libraries(mikaddr miksys)

# miknet datagrams
add_library(mikgram mikgram.c)

# miknet messages
add_library(mikmsg mikmsg.c)
target_link_libraries(mikmsg mikgram)

# miknet network interface
add_library(mikstation mikstation.c)
target_link_libraries(mikstation mikgram miksys)

# miknet node/user interface
add_library(miknode miknode.c)
target_link_libraries(miknode mikaddr mikmsg mikgram mikstation miksys miktime)
