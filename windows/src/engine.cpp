// SPDX-License-Identifier: MIT OR Apache-2.0
#include "engine.h"

#include <buuz.h>

namespace buuz {

std::u16string utf8_to_utf16(const char* s, size_t len) {
    std::u16string out;
    if (!s) return out;
    out.reserve(len);
    size_t i = 0;
    while (i < len) {
        const uint8_t b = static_cast<uint8_t>(s[i]);
        uint32_t cp = 0xFFFD;
        size_t n = 1;
        if (b < 0x80) {
            cp = b;
        } else if (b >= 0xC2 && b < 0xE0 && i + 1 < len && (s[i + 1] & 0xC0) == 0x80) {
            cp = ((b & 0x1Fu) << 6) | (s[i + 1] & 0x3Fu);
            n = 2;
        } else if (b >= 0xE0 && b < 0xF0 && i + 2 < len && (s[i + 1] & 0xC0) == 0x80 &&
                   (s[i + 2] & 0xC0) == 0x80) {
            cp = ((b & 0x0Fu) << 12) | ((s[i + 1] & 0x3Fu) << 6) | (s[i + 2] & 0x3Fu);
            n = 3;
            if (cp < 0x800 || (cp >= 0xD800 && cp < 0xE000)) {
                cp = 0xFFFD;
            }
        } else if (b >= 0xF0 && b < 0xF5 && i + 3 < len && (s[i + 1] & 0xC0) == 0x80 &&
                   (s[i + 2] & 0xC0) == 0x80 && (s[i + 3] & 0xC0) == 0x80) {
            cp = ((b & 0x07u) << 18) | ((s[i + 1] & 0x3Fu) << 12) | ((s[i + 2] & 0x3Fu) << 6) |
                 (s[i + 3] & 0x3Fu);
            n = 4;
            if (cp < 0x10000 || cp > 0x10FFFF) cp = 0xFFFD;
        }
        if (cp >= 0x10000) {
            cp -= 0x10000;
            out.push_back(static_cast<char16_t>(0xD800 + (cp >> 10)));
            out.push_back(static_cast<char16_t>(0xDC00 + (cp & 0x3FF)));
        } else {
            out.push_back(static_cast<char16_t>(cp));
        }
        i += n;
    }
    return out;
}

Engine::Engine(const std::string& config)
    : composer_(buuz_composer_new(
          config.empty() ? nullptr : reinterpret_cast<const uint8_t*>(config.data()),
          config.size(), nullptr)) {}

Engine::~Engine() { buuz_composer_free(composer_); }

bool Engine::composing() const { return composer_ && buuz_is_composing(composer_); }

Edit Engine::process(const KeyInput& key) {
    Edit edit;
    if (!composer_) return edit;
    BuuzOutcome out{};
    if (!buuz_process_key(composer_, key.kind, key.ch, &out)) return edit;
    edit.consumed = out.consumed;
    edit.delete_before = out.delete_before;
    // Read the commit first: the next call on the handle may reuse its buffer.
    edit.commit = utf8_to_utf16(out.commit, out.commit_len);
    size_t len = 0;
    const char* pre = buuz_preedit(composer_, &len);
    edit.preedit = utf8_to_utf16(pre, len);
    edit.cursor = buuz_cursor(composer_);
    if (edit.cursor > edit.preedit.size()) edit.cursor = static_cast<uint32_t>(edit.preedit.size());
    edit.composing = buuz_is_composing(composer_);
    return edit;
}

void Engine::cancel() {
    if (composer_) buuz_cancel(composer_);
}

}  // namespace buuz
