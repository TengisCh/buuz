// SPDX-License-Identifier: MIT OR Apache-2.0
// Tests for the pure shell logic, linked against the real Rust core.
#include <cstdio>
#include <cstdlib>
#include <string>

#include "engine.h"
#include "keymap.h"

using namespace buuz;

static int failures = 0;
#define CHECK(cond)                                                      \
    do {                                                                 \
        if (!(cond)) {                                                   \
            std::fprintf(stderr, "%s:%d: CHECK(%s)\n", __FILE__, __LINE__, #cond); \
            failures++;                                                  \
        }                                                                \
    } while (0)

// Type US keys the way TSF delivers them: virtual-key code plus modifiers.
static Edit key(Engine& e, uint32_t vk, Modifiers m = {}) {
    return e.process(classify(vk, m));
}
static Edit type(Engine& e, const char* latin) {
    Edit last;
    for (const char* p = latin; *p; ++p) {
        Modifiers m;
        uint32_t vk = static_cast<unsigned char>(*p);
        if (vk >= 'a' && vk <= 'z') vk -= 32;
        last = key(e, vk, m);
    }
    return last;
}

static void test_translate() {
    CHECK(translate_us('A', false, false) == 'a');
    CHECK(translate_us('A', true, false) == 'A');
    CHECK(translate_us('A', false, true) == 'A');
    CHECK(translate_us('A', true, true) == 'a');  // Shift cancels Caps Lock
    CHECK(translate_us('1', true, false) == '!');
    CHECK(translate_us('1', false, true) == '1');  // Caps Lock leaves digits alone
    CHECK(translate_us(0xBE, false, false) == '.');
    CHECK(translate_us(0xBF, true, false) == '?');
    CHECK(translate_us(0x70, false, false) == 0);  // F1
}

static void test_classify() {
    Modifiers none;
    CHECK(classify('G', none).kind == 0 && classify('G', none).ch == 'g');
    CHECK(classify(0x20, none).kind == 8);
    CHECK(classify(0x08, none).kind == 1);
    CHECK(classify(0x10, none).kind == 7);
    CHECK(classify(0xBE, none).kind == 10);
    Modifiers ctrl;
    ctrl.ctrl = true;
    CHECK(classify('A', ctrl).kind == 12);  // Ctrl+A goes to the app
    CHECK(!is_typing_key('A', ctrl));
    CHECK(is_typing_key('A', none));
    CHECK(!is_typing_key(0x0D, none));
}

static void test_utf16() {
    CHECK(utf8_to_utf16("sain", 4) == u"sain");
    const char* s = "\xD0\xB3\xD1\x8D\xD1\x80";  // гэр
    CHECK(utf8_to_utf16(s, 6) == u"гэр");
    const char* emoji = "\xF0\x9F\xA5\x9F";  // U+1F95F
    CHECK(utf8_to_utf16(emoji, 4) == u"\xD83E\xDD5F");
    CHECK(utf8_to_utf16("\xC0\x80", 2) == u"��");  // overlong
}

static void test_typing() {
    Engine e;
    CHECK(e.ok());
    Edit ed = type(e, "geree");
    CHECK(ed.consumed);
    CHECK(ed.composing);
    CHECK(ed.preedit == u"гэрээ");  // гэрээ
    CHECK(ed.cursor == 5);
    // Space commits the word and goes on to the app.
    ed = key(e, 0x20);
    CHECK(!ed.consumed);
    CHECK(ed.commit == u"гэрээ");
    CHECK(!ed.composing);
    CHECK(ed.preedit.empty());
    CHECK(!e.composing());
}

static void test_edit_keys() {
    Engine e;
    type(e, "sain");
    Edit ed = key(e, 0x08);  // Backspace removes one letter
    CHECK(ed.consumed);
    CHECK(ed.preedit == u"сай");  // сай
    ed = key(e, 0x1B);  // Esc: no commit loss, preedit stays Cyrillic or turns Latin
    CHECK(ed.consumed || !ed.composing);
    e.cancel();
    CHECK(!e.composing());
    // Enter commits and passes through.
    type(e, "sain");
    ed = key(e, 0x0D);
    CHECK(!ed.consumed);
    CHECK(ed.commit == u"сайн");
}

static void test_fail_open() {
    Engine bad("this is not a config\n\x01");
    if (!bad.ok()) {
        Edit ed = bad.process(classify('A', Modifiers{}));
        CHECK(!ed.consumed);  // a broken engine never eats keys
    }
}

int main() {
    test_translate();
    test_classify();
    test_utf16();
    test_typing();
    test_edit_keys();
    test_fail_open();
    if (failures) {
        std::fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    std::puts("windows shell logic: all tests passed");
    return 0;
}
