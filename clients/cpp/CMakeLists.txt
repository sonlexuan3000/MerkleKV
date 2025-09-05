cmake_minimum_required(VERSION 3.16)
project(merklekv-cpp VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Options
option(MERKLEKV_BUILD_TESTS "Build tests" ON)
option(MERKLEKV_BUILD_EXAMPLES "Build examples" ON)

# Find required packages
find_package(Threads REQUIRED)

# Create library (header-only)
add_library(merklekv-cpp INTERFACE)
add_library(merklekv::client ALIAS merklekv-cpp)

target_include_directories(merklekv-cpp INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

target_compile_features(merklekv-cpp INTERFACE cxx_std_17)
target_link_libraries(merklekv-cpp INTERFACE Threads::Threads)

# Platform-specific networking libraries
if(WIN32)
    target_link_libraries(merklekv-cpp INTERFACE ws2_32)
else()
    # Unix-like systems (Linux, macOS)
    # No additional libraries needed
endif()

# Installation
include(GNUInstallDirs)
install(TARGETS merklekv-cpp EXPORT merklekv-cpp-targets)
install(DIRECTORY include/ DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

# Generate and install config files
include(CMakePackageConfigHelpers)

configure_package_config_file(
    cmake/merklekv-cpp-config.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/merklekv-cpp-config.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/merklekv-cpp
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/merklekv-cpp-config-version.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY AnyNewerVersion
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/merklekv-cpp-config.cmake
    ${CMAKE_CURRENT_BINARY_DIR}/merklekv-cpp-config-version.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/merklekv-cpp
)

install(EXPORT merklekv-cpp-targets
    FILE merklekv-cpp-targets.cmake
    NAMESPACE merklekv::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/merklekv-cpp
)

# Tests
if(MERKLEKV_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# Examples
if(MERKLEKV_BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()
