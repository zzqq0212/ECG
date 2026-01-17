idf_component_register(SRCS "hello_world_main.c"
                    INCLUDE_DIRS "")


target_compile_options(${COMPONENT_LIB} PRIVATE -fsanitize=kernel-address)
target_compile_options(${COMPONENT_LIB} PRIVATE -fasan-shadow-offset=0x3780A400)
target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-stack=1")
target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-globals=1")
#target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-instrument-allocas=1")
target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-instrument-reads=1")
target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-instrument-writes=1")
target_compile_options(${COMPONENT_LIB} PRIVATE "SHELL:--param asan-instrumentation-with-call-threshold=0")
