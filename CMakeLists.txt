cmake_minimum_required(VERSION 3.16)
project(webview)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)
set(CMAKE_SUPPRESS_REGENERATION true)
set(CMAKE_SKIP_INSTALL_ALL_DEPENDENCY true)

add_library(webview SHARED
            ./cxx/library.h
            ./cxx/webview.cpp
            ./cxx/webview.h
            ./cxx/runtime.cpp
            ./cxx/runtime.h
            ./cxx/subprocess.h
            ./cxx/subprocess.cpp
            ./cxx/util.cpp
            ./cxx/util.h
            ./cxx/request.h
            ./cxx/request.cpp)

# You need to manually create the directory and copy the CEF source code to this directory.
set(THIRD_PARTY_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party")

include_directories("${THIRD_PARTY_DIR}/cef")

target_link_directories(webview PRIVATE
						"${THIRD_PARTY_DIR}/cef/${CMAKE_BUILD_TYPE}"
						"${THIRD_PARTY_DIR}/cef/libcef_dll_wrapper/${CMAKE_BUILD_TYPE}")


if(MSVC)
    add_compile_definitions(WIN32)
elseif(APPLE)
    add_compile_definitions(MACOS)
elseif(UNIX AND NOT APPLE)
    add_compile_definitions(LINUX)
endif()
