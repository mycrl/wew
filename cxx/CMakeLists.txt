cmake_minimum_required(VERSION 3.16)
project(webview)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# TODO: Doing so may prevent some dependencies from being built 
# or installed correctly, so use with caution.
set(CMAKE_SUPPRESS_REGENERATION true)
set(CMAKE_SKIP_INSTALL_ALL_DEPENDENCY true)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug")
endif()

set(THIRD_PARTY_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party")

add_library(webview SHARED
            ./webview.cpp
            ./webview.h
            ./app.cpp
            ./app.h
            ./page.cpp
            ./page.h
            ./render.cpp
            ./render.h
            ./display.cpp
            ./display.h
            ./control.cpp
            ./control.h
            ./scheme_handler.h
            ./scheme_handler.cpp)

include_directories("${THIRD_PARTY_DIR}/cef")
target_link_directories(webview PRIVATE
						"${THIRD_PARTY_DIR}/cef/${CMAKE_BUILD_TYPE}"
						"${THIRD_PARTY_DIR}/cef/libcef_dll_wrapper/${CMAKE_BUILD_TYPE}")
target_link_libraries(webview
					  libcef
					  libcef_dll_wrapper
                      cef_sandbox
                      winmm
                      delayimp)

if(MSVC)
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
        set_property(TARGET webview PROPERTY MSVC_RUNTIME_LIBRARY MultiThreaded)
    else()
        set_property(TARGET webview PROPERTY MSVC_RUNTIME_LIBRARY MultiThreadedDebug)
    endif()
endif()

if(MSVC)
    add_compile_definitions(WIN32
                            _WINDOWS
                            __STDC_CONSTANT_MACROS
                            __STDC_FORMAT_MACROS
                            _WIN32
                            UNICODE
                            _UNICODE
                            WINVER=0x0A00
                            _WIN32_WINNT=0x0A00
                            NTDDI_VERSION=NTDDI_WIN10_FE
                            NOMINMAX
                            WIN32_LEAN_AND_MEAN
                            _HAS_EXCEPTIONS=0
                            PSAPI_VERSION=1
                            _HAS_ITERATOR_DEBUGGING=0
                            CEF_USE_ATL
                            CEF_USE_SANDBOX)
elseif(APPLE)
    add_compile_definitions(MACOS)
elseif(UNIX AND NOT APPLE)
    add_compile_definitions(LINUX)
endif()
