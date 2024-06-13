idf_component_register(SRCS "kcov.c"
                    INCLUDE_DIRS .)
target_compile_options(${COMPONENT_LIB} PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(${COMPONENT_LIB} PRIVATE "-fno-sanitize=kernel-address")

