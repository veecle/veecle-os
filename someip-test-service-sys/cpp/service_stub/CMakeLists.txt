cmake_minimum_required(VERSION 3.28 FATAL_ERROR)
project(someip-test-service DESCRIPTION "No-op implementation of a test service for platforms that does not support vsomeip")

add_library(${CMAKE_PROJECT_NAME} SHARED lib.cpp)
install(
    TARGETS ${CMAKE_PROJECT_NAME}
    LIBRARY DESTINATION "${INSTALL_LIB_DIR}" COMPONENT shlib
    RUNTIME DESTINATION "${INSTALL_BIN_DIR}" COMPONENT bin
)