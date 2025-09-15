cmake_minimum_required(VERSION 3.28 FATAL_ERROR)
project(someip-test-service DESCRIPTION "SOME/IP test service")

set(CMAKE_CXX_STANDARD 20)

# Because we re-use internal logger of CommonAPI.
add_definitions(-DCOMMONAPI_INTERNAL_COMPILATION)

# Because we want to avoid finding a wrong package.
set(FIND_ONLY_IN_CMAKE_PREFIX_PATH
    NO_PACKAGE_ROOT_PATH
    NO_SYSTEM_ENVIRONMENT_PATH
    NO_CMAKE_PACKAGE_REGISTRY
    NO_CMAKE_SYSTEM_PATH
    NO_CMAKE_INSTALL_PREFIX
    NO_CMAKE_SYSTEM_PACKAGE_REGISTRY)
find_package(CommonAPI 3.2.4 REQUIRED CONFIG ${FIND_ONLY_IN_CMAKE_PREFIX_PATH})
find_package(CommonAPI-SomeIP 3.2.4 REQUIRED CONFIG ${FIND_ONLY_IN_CMAKE_PREFIX_PATH})

add_library(${CMAKE_PROJECT_NAME} SHARED src/lib.cpp)
add_library(${CMAKE_PROJECT_NAME}-someip SHARED
    gen/v0/test/TestServiceSomeIPDeployment.cpp
    gen/v0/test/TestServiceSomeIPProxy.cpp
    gen/v0/test/TestServiceSomeIPStubAdapter.cpp)

target_include_directories(${CMAKE_PROJECT_NAME} PRIVATE gen src)
target_include_directories(${CMAKE_PROJECT_NAME}-someip PRIVATE gen)

target_link_libraries(${CMAKE_PROJECT_NAME} CommonAPI)
target_link_libraries(${CMAKE_PROJECT_NAME}-someip CommonAPI-SomeIP)

install(
    TARGETS ${CMAKE_PROJECT_NAME}
    LIBRARY DESTINATION "${INSTALL_LIB_DIR}" COMPONENT shlib
    RUNTIME DESTINATION "${INSTALL_BIN_DIR}" COMPONENT bin
)
install(
    TARGETS ${CMAKE_PROJECT_NAME}-someip
    LIBRARY DESTINATION "${INSTALL_LIB_DIR}" COMPONENT shlib
    RUNTIME DESTINATION "${INSTALL_BIN_DIR}" COMPONENT bin
)
