# The following lines of boilerplate have to be in your project's
# CMakeLists in this exact order for cmake to work correctly
cmake_minimum_required(VERSION 3.16)

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(hello_world)

idf_build_set_property(COMPILE_OPTIONS "-fsanitize-coverage=trace-pc,trace-cmp" APPEND)
target_compile_options(__idf_soc PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_hal PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_xtensa PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_esp_system PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_esp_hw_support PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_esp_rom PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_spi_flash PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
target_compile_options(__idf_freertos PRIVATE "-fno-sanitize-coverage=trace-pc,trace-cmp")
