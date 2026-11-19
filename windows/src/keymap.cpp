// SPDX-License-Identifier: MIT OR Apache-2.0
#include "keymap.h"

#include <buuz.h>

namespace buuz {

namespace {

// OEM keys on a US keyboard: {vk, plain, shifted}.
struct Oem {
    uint32_t vk;
    char plain;
    char shifted;
};
constexpr Oem kOem[] = {
    {0xBA, ';', ':'}, {0xBB, '=', '+'}, {0xBC, ',', '<'}, {0xBD, '-', '_'},
    {0xBE, '.', '>'}, {0xBF, '/', '?'}, {0xC0, '`', '~'}, {0xDB, '[', '{'},
    {0xDC, '\\', '|'}, {0xDD, ']', '}'}, {0xDE, '\'', '"'},
    // The extra key on ISO keyboards. macOS has no fixed character for it
    // (it follows the layout), so use '<' and '>' as a US-ISO layout does.
    {0xE2, '<', '>'},
};
// Numpad operators: VK_MULTIPLY, VK_ADD, (VK_SEPARATOR 0x6C gives nothing),
// VK_SUBTRACT, VK_DECIMAL, VK_DIVIDE. Shift does not change them.
constexpr char kNumpadOps[] = {'*', '+', 0, '-', '.', '/'};
constexpr char kShiftedDigits[] = ")!@#$%^&*(";

}  // namespace

uint32_t translate_us(uint32_t vk, bool shift, bool caps) {
    if (vk >= 'A' && vk <= 'Z') {
        const bool upper = shift != caps;
        return upper ? vk : vk + ('a' - 'A');
    }
    if (vk >= '0' && vk <= '9') {
        return shift ? static_cast<uint32_t>(kShiftedDigits[vk - '0']) : vk;
    }
    // Numpad keys. With NumLock off Windows sends VK_HOME, VK_END and so on
    // instead of VK_NUMPAD*, so NumLock needs no special handling here.
    if (vk >= 0x60 && vk <= 0x69) return '0' + (vk - 0x60);
    if (vk >= 0x6A && vk <= 0x6F) return static_cast<uint32_t>(kNumpadOps[vk - 0x6A]);
    for (const Oem& k : kOem) {
        if (k.vk == vk) return static_cast<uint32_t>(shift ? k.shifted : k.plain);
    }
    return 0;
}

KeyInput classify(uint32_t vk, const Modifiers& mods) {
    switch (vk) {
        case VK_SHIFT_:
        case VK_CAPITAL_:
            return {BUUZ_KEY_MODIFIER, 0};
        case VK_CONTROL_:
        case VK_MENU_:
            return {BUUZ_KEY_OTHER, 0};
        default:
            break;
    }
    if (mods.ctrl || mods.alt) return {BUUZ_KEY_OTHER, 0};
    switch (vk) {
        case VK_BACK_: return {BUUZ_KEY_BACKSPACE, 0};
        case VK_DELETE_: return {BUUZ_KEY_DELETE, 0};
        case VK_LEFT_: return {BUUZ_KEY_LEFT, 0};
        case VK_RIGHT_: return {BUUZ_KEY_RIGHT, 0};
        case VK_HOME_: return {BUUZ_KEY_HOME, 0};
        case VK_END_: return {BUUZ_KEY_END, 0};
        case VK_SPACE_: return {BUUZ_KEY_SPACE, ' '};
        case VK_ESCAPE_: return {BUUZ_KEY_ESCAPE, 0};
        case VK_TAB_: return {BUUZ_KEY_TAB, 0};
        default: break;
    }
    const uint32_t ch = translate_us(vk, mods.shift, mods.caps);
    if (ch == 0) return {BUUZ_KEY_OTHER, 0};
    const bool letter = (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
    return {static_cast<uint32_t>(letter ? BUUZ_KEY_CHAR : BUUZ_KEY_SYMBOL), ch};
}

bool is_typing_key(uint32_t vk, const Modifiers& mods) {
    if (mods.ctrl || mods.alt) return false;
    return translate_us(vk, mods.shift, mods.caps) != 0;
}

}  // namespace buuz
