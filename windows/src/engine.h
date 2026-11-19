// SPDX-License-Identifier: MIT OR Apache-2.0
// The shell's view of the Rust core: one key in, one edit plan out. No
// Windows headers, so tests run on any system.
#pragma once

#include <cstddef>
#include <cstdint>
#include <string>

#include "keymap.h"

struct BuuzComposer;

namespace buuz {

// What to do in the text field after one key. Apply in this order: delete
// `delete_before` characters before the caret, replace the composition with
// `commit`, end it, then show `preedit` as a new underlined composition.
struct Edit {
    bool consumed = false;  // false: pass the key on to the application
    uint32_t delete_before = 0;
    std::u16string commit;
    std::u16string preedit;
    uint32_t cursor = 0;  // in UTF-16 units inside `preedit`
    bool composing = false;
};

class Engine {
public:
    // `config` is the portable config text, or empty for the smart defaults.
    explicit Engine(const std::string& config = std::string());
    ~Engine();
    Engine(const Engine&) = delete;
    Engine& operator=(const Engine&) = delete;

    // False when the core could not be created. Then `process` passes every
    // key through: the input method fails open.
    bool ok() const { return composer_ != nullptr; }
    bool composing() const;

    // One key down. On any error the key passes through untouched.
    Edit process(const KeyInput& key);
    // Drop the composition without a commit (focus lost, app closed it).
    void cancel();

private:
    BuuzComposer* composer_;
};

// UTF-8 to UTF-16. Bad bytes become U+FFFD. Mongolian Cyrillic is in the
// Basic Multilingual Plane, but the function handles every code point.
std::u16string utf8_to_utf16(const char* s, size_t len);

}  // namespace buuz
