// SPDX-License-Identifier: MIT OR Apache-2.0
// Pure key logic for the Windows shell. No Windows headers, so it builds and
// runs in tests on any system.
#pragma once

#include <cstdint>

namespace buuz {

// Windows virtual-key codes used by the shell (same values as winuser.h).
enum Vk : uint32_t {
    VK_BACK_ = 0x08,
    VK_TAB_ = 0x09,
    VK_RETURN_ = 0x0D,
    VK_SHIFT_ = 0x10,
    VK_CONTROL_ = 0x11,
    VK_MENU_ = 0x12,
    VK_CAPITAL_ = 0x14,
    VK_ESCAPE_ = 0x1B,
    VK_SPACE_ = 0x20,
    VK_END_ = 0x23,
    VK_HOME_ = 0x24,
    VK_LEFT_ = 0x25,
    VK_RIGHT_ = 0x27,
    VK_DELETE_ = 0x2E,
};

struct Modifiers {
    bool shift = false;
    bool caps = false;
    bool ctrl = false;
    bool alt = false;  // Alt without Ctrl (AltGr counts as Ctrl+Alt)
};

// One key for buuz_process_key. `kind` is a BUUZ_KEY_* value.
struct KeyInput {
    uint32_t kind;
    uint32_t ch;
};

// The character a US QWERTY keyboard gives for `vk`, or 0 if the key is not
// a printable key. Buuz is a phonetic Latin layout, so the shell reads keys
// as US QWERTY whatever layout is installed.
uint32_t translate_us(uint32_t vk, bool shift, bool caps);

// Map one key down event to a BuuzKey. Ctrl and Alt chords are `OTHER`: they
// end the composition and go to the application.
KeyInput classify(uint32_t vk, const Modifiers& mods);

// True when the key makes a character (a letter or symbol), so the shell
// must offer it to the engine even when no composition is active.
bool is_typing_key(uint32_t vk, const Modifiers& mods);

}  // namespace buuz
