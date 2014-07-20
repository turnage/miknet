include_directories(include)

# miknet logging
add_library(miklogger miklogger.c)

# miknet memory management and testing
add_library(mikmemapi mikmemapi.c)
add_library(mikmemshield mikmemshield.c)
target_link_libraries(mikmemshield mikmemapi)
add_library(mikmemmock mikmemmock.c)
