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

# Pure logic: key mapping, the engine wrapper and the edit plan. No Windows headers.
add_library(buuz_shell STATIC src/keymap.cpp src/engine.cpp src/edit_plan.cpp)
target_include_directories(buuz_shell PUBLIC src)
target_link_libraries(buuz_shell PUBLIC buuz_ffi)

# Settings app logic: config edits, state and strings. No Windows headers.
set(BUUZ_STRINGS_CPP "${CMAKE_CURRENT_BINARY_DIR}/strings_data.cpp")
set(BUUZ_STRINGS_MN "${CMAKE_CURRENT_SOURCE_DIR}/app/strings_mn.tsv")
set(BUUZ_STRINGS_EN "${CMAKE_CURRENT_SOURCE_DIR}/app/strings_en.tsv")
add_custom_command(
  OUTPUT "${BUUZ_STRINGS_CPP}"
  COMMAND ${CMAKE_COMMAND} -DMN=${BUUZ_STRINGS_MN} -DEN=${BUUZ_STRINGS_EN}
          -DOUT=${BUUZ_STRINGS_CPP} -P "${CMAKE_CURRENT_SOURCE_DIR}/cmake_strings.cmake"
  DEPENDS "${BUUZ_STRINGS_MN}" "${BUUZ_STRINGS_EN}" cmake_strings.cmake
  VERBATIM)
add_library(buuz_app_logic STATIC app/conf.cpp app/app_state.cpp app/strings.cpp "${BUUZ_STRINGS_CPP}")
target_include_directories(buuz_app_logic PUBLIC app)

enable_testing()
add_executable(app_test tests/app_test.cpp)
target_link_libraries(app_test PRIVATE buuz_app_logic)
add_test(NAME app_test COMMAND app_test)
add_executable(engine_test tests/engine_test.cpp)
target_link_libraries(engine_test PRIVATE buuz_shell)
add_test(NAME engine_test COMMAND engine_test)
add_executable(edit_plan_test tests/edit_plan_test.cpp)
target_link_libraries(edit_plan_test PRIVATE buuz_shell)
add_test(NAME edit_plan_test COMMAND edit_plan_test)

if(WIN32)
  add_library(buuz_tsf SHARED
    src/text_service.cpp src/registration.cpp src/display_attributes.cpp src/guids.cpp buuz_tsf.def)
  target_compile_definitions(buuz_tsf PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN)
  target_link_libraries(buuz_tsf PRIVATE buuz_shell ole32 oleaut32 uuid advapi32 user32)
  # ARM64X: the ARM64 and x64 builds get a suffix (_arm64, _x64). A small
  # forwarder named buuz_tsf.dll loads the right one (windows/arm64x).
  set(BUUZ_DLL_SUFFIX "" CACHE STRING "Suffix for the DLL name, e.g. _x64")
  if(BUUZ_DLL_SUFFIX)
    set_property(TARGET buuz_tsf PROPERTY OUTPUT_NAME "buuz_tsf${BUUZ_DLL_SUFFIX}")
  endif()
  if(MSVC)
    # Static C runtime: nothing else to install, small load time.
    set_property(TARGET buuz_tsf buuz_shell engine_test edit_plan_test buuz_app_logic app_test PROPERTY
      MSVC_RUNTIME_LIBRARY "MultiThreaded")
    target_link_options(buuz_tsf PRIVATE /DEBUG /OPT:REF /OPT:ICF)
  endif()

  # Settings and onboarding app: plain Win32, runs apart from the DLL.
  add_executable(buuz_settings WIN32 app/main.cpp src/guids.cpp
    $<$<CXX_COMPILER_ID:MSVC>:app/buuz_settings.manifest>)
  target_compile_definitions(buuz_settings PRIVATE UNICODE _UNICODE WIN32_LEAN_AND_MEAN)
  target_include_directories(buuz_settings PRIVATE src)
  target_link_libraries(buuz_settings PRIVATE buuz_app_logic ole32 oleaut32 uuid user32 gdi32 shell32 comctl32)
  if(MSVC)
    set_property(TARGET buuz_settings PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
    target_link_options(buuz_settings PRIVATE /DEBUG /OPT:REF /OPT:ICF)
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
