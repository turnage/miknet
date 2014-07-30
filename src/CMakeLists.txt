include_directories(include)

# miknet logging
add_library(miklog miklog.c)

# miknet id and signature generation
add_library(mikid mikid.c)
target_link_libraries(mikid miklog)
