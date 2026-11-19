# SPDX-License-Identifier: MIT OR Apache-2.0
cmake_minimum_required(VERSION 3.20)
project(buuz_windows CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(MSVC)
  add_compile_options(/W4 /WX /utf-8)
else()
  add_compile_options(-Wall -Wextra -Werror)
endif()

# The Rust static library from ffi/ (run `cargo build --release --locked`
# in ffi/ first). Set -DBUUZ_FFI_LIB=<path> to use another build.
set(BUUZ_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../ffi")
if(WIN32)
  set(_ffi_name buuz_ffi.lib)
else()
  set(_ffi_name libbuuz_ffi.a)
endif()
set(BUUZ_FFI_LIB "${BUUZ_FFI_DIR}/target/release/${_ffi_name}" CACHE FILEPATH "buuz-ffi static library")

add_library(buuz_ffi STATIC IMPORTED)
set_target_properties(buuz_ffi PROPERTIES
  IMPORTED_LOCATION "${BUUZ_FFI_LIB}"
  INTERFACE_INCLUDE_DIRECTORIES "${BUUZ_FFI_DIR}/include")
if(WIN32)
  # System libraries the Rust standard library needs on Windows.
  set_property(TARGET buuz_ffi PROPERTY INTERFACE_LINK_LIBRARIES
    ws2_32 userenv ntdll bcrypt advapi32 kernel32 synchronization)
elseif(UNIX)
  set_property(TARGET buuz_ffi PROPERTY INTERFACE_LINK_LIBRARIES pthread dl m)
endif()

# Pure logic: key mapping and the engine wrapper. No Windows headers.
add_library(buuz_shell STATIC src/keymap.cpp src/engine.cpp)
target_include_directories(buuz_shell PUBLIC src)
target_link_libraries(buuz_shell PUBLIC buuz_ffi)

enable_testing()
add_executable(engine_test tests/engine_test.cpp)
target_link_libraries(engine_test PRIVATE buuz_shell)
add_test(NAME engine_test COMMAND engine_test)

if(WIN32)
  add_library(buuz_tsf SHARED
    src/text_service.cpp src/registration.cpp src/display_attributes.cpp src/guids.cpp buuz_tsf.def)
  target_compile_definitions(buuz_tsf PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN)
  target_link_libraries(buuz_tsf PRIVATE buuz_shell ole32 oleaut32 uuid advapi32 user32)
  if(MSVC)
    # Static C runtime: nothing else to install, small load time.
    set_property(TARGET buuz_tsf buuz_shell engine_test PROPERTY
      MSVC_RUNTIME_LIBRARY "MultiThreaded")
    target_link_options(buuz_tsf PRIVATE /DEBUG /OPT:REF /OPT:ICF)
  endif()

  # End-to-end typing test. It needs the DLL registered (regsvr32) and an
  # interactive desktop, so it does not run under plain `ctest`.
  add_executable(e2e_typing tests/e2e_typing.cpp src/guids.cpp)
  target_compile_definitions(e2e_typing PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN)
  target_link_libraries(e2e_typing PRIVATE ole32 oleaut32 uuid user32)
  if(MSVC)
    set_property(TARGET e2e_typing PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
  endif()
endif()
