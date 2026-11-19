// SPDX-License-Identifier: MIT OR Apache-2.0
// Tests for the pure shell logic, linked against the real Rust core.
#include <cstdio>
#include <chrono>
#include <cstdlib>
#include <fstream>
#include <sstream>
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

static void test_bad_config_fails_open() {
    Engine bad(std::string("\xff\xfe", 2));
    CHECK(!bad.ok());
    CHECK(!bad.composing());
    Edit ed = bad.process(classify('A', Modifiers{}));
    CHECK(!ed.consumed);  // a broken engine never eats keys
}

static void test_null_commit_is_empty() {
    CHECK(utf8_to_utf16(nullptr, 3).empty());
}

static void test_truncated_utf8() {
    CHECK(utf8_to_utf16("\xD0", 1) == u"\uFFFD");
}

static const char* kAutocorrect = "buuz_config = 1\n[options]\nautocorrect = true\n";

// Same case as `the_word_list_reaches_the_composer` in ffi/tests/abi.rs.
static void test_word_list_reaches_engine() {
    const std::string list = "\xd0\xb4\xd1\x8d\xd0\xbb\xd0\xb3\xd2\xaf\xd2\xaf\xd1\x80\t100\n";  // дэлгүүр
    WordList words(list);
    CHECK(words.ok());
    int loads = 0;
    Engine e(kAutocorrect, [&]() {
        loads++;
        return words.get();
    });
    CHECK(e.ok());
    CHECK(loads == 0);  // not at construction: the first key loads it
    type(e, "delgur");
    CHECK(loads == 1);
    Edit ed = key(e, 0x20);
    CHECK(ed.commit == u"дэлгүүр");  // дэлгүүр
    type(e, "delgur");
    key(e, 0x20);
    CHECK(loads == 1);  // once, never again on the key path
}

// Smart mode, default config: the list picks ий over ы (issue #66).
static void test_word_list_chooses_ii() {
    const std::string list =
        "\xd1\x81\xd1\x83\xd1\x80\xd0\xb3\xd1\x83\xd1\x83\xd0\xbb\xd0\xb8\xd0\xb9\xd0\xbd\t100\n";  // сургуулийн
    WordList words(list);
    Engine with("", [&]() { return words.get(); });
    Engine without;
    const Edit a = type(with, "surguuliin");
    const Edit b = type(without, "surguuliin");
    CHECK(a.preedit == u"сургуулийн");  // сургуулийн
    CHECK(b.preedit != a.preedit);
}

static void test_bad_word_list_still_types() {
    const std::string garbage("\xff\xfe", 2);
    WordList bad(garbage);
    CHECK(!bad.ok());
    CHECK(bad.get() == nullptr);
    Engine e1("", [&]() { return bad.get(); });
    CHECK(e1.ok());
    CHECK(type(e1, "geree").preedit == u"гэрээ");  // гэрээ
    Engine e2("", []() -> const BuuzWordList* { return nullptr; });
    CHECK(type(e2, "geree").preedit == u"гэрээ");
}

// Prints how long the real list takes to parse. Skipped without the file.
static void test_real_word_list_parse_time() {
#ifdef BUUZ_WORDLIST_PATH
    std::ifstream f(BUUZ_WORDLIST_PATH, std::ios::binary);
    if (!f) return;
    std::stringstream ss;
    ss << f.rdbuf();
    const std::string text = ss.str();
    const auto t0 = std::chrono::steady_clock::now();
    WordList words(text);
    const auto t1 = std::chrono::steady_clock::now();
    CHECK(words.ok());
    std::printf("mn.txt: %zu bytes parsed in %.1f ms\n", text.size(),
                std::chrono::duration<double, std::milli>(t1 - t0).count());
#endif
}

int main() {
    test_translate();
    test_classify();
    test_utf16();
    test_typing();
    test_edit_keys();
    test_bad_config_fails_open();
    test_null_commit_is_empty();
    test_truncated_utf8();
    test_word_list_reaches_engine();
    test_word_list_chooses_ii();
    test_bad_word_list_still_types();
    test_real_word_list_parse_time();
    if (failures) {
        std::fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    std::puts("windows shell logic: all tests passed");
    return 0;
}
