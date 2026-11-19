// SPDX-License-Identifier: MIT OR Apache-2.0
// Tests for the pure edit plan, linked against the real Rust core.
#include <cstdio>

#include "edit_plan.h"
#include "engine.h"
#include "keymap.h"

using namespace buuz;

static int failures = 0;
#define CHECK(cond)                                                                \
    do {                                                                           \
        if (!(cond)) {                                                             \
            std::fprintf(stderr, "%s:%d: CHECK(%s)\n", __FILE__, __LINE__, #cond); \
            failures++;                                                            \
        }                                                                          \
    } while (0)

static Edit key(Engine& e, uint32_t vk) { return e.process(classify(vk, Modifiers{})); }
static Edit type(Engine& e, const char* latin) {
    Edit last;
    for (const char* p = latin; *p; ++p) {
        uint32_t vk = static_cast<unsigned char>(*p);
        if (vk >= 'a' && vk <= 'z') vk -= 32;
        last = key(e, vk);
    }
    return last;
}

static void test_pass_through_without_change() {
    Edit ed;  // consumed = false, nothing to commit or delete
    const EditPlan p = plan(ed, false);
    CHECK(!p.touch_text);
    CHECK(!p.end_composition);
    CHECK(!p.start_composition);
}

static void test_commit_then_pass() {
    Engine e;
    type(e, "geree");
    const Edit ed = key(e, 0x20);
    CHECK(!ed.consumed);
    const EditPlan p = plan(ed, true);
    CHECK(p.touch_text);
    CHECK(p.commit == u"гэрээ");
    CHECK(p.end_composition);
    CHECK(!p.start_composition);
    CHECK(p.preedit.empty());
}

static void test_preedit_starts_composition() {
    Engine e;
    const Edit ed = type(e, "s");
    const EditPlan p = plan(ed, false);
    CHECK(p.touch_text);
    CHECK(p.start_composition);
    CHECK(!p.end_composition);
    CHECK(p.preedit == u"с");
    CHECK(p.caret_utf16 == 1);
}

static void test_empty_preedit_ends_composition() {
    Engine e;
    type(e, "s");
    const Edit ed = key(e, 0x08);
    const EditPlan p = plan(ed, true);
    CHECK(p.touch_text);
    CHECK(p.end_composition);
    CHECK(!p.start_composition);
    CHECK(p.preedit.empty());
}

static void test_cursor_clamped() {
    Edit ed;
    ed.consumed = true;
    ed.composing = true;
    ed.preedit = u"сай";
    ed.cursor = 99;
    ed.delete_before = 2;
    const EditPlan p = plan(ed, true);
    CHECK(p.caret_utf16 == 3);
    CHECK(p.delete_utf16 == 2);
    CHECK(!p.start_composition);  // the composition goes on
    CHECK(!p.end_composition);
}

int main() {
    test_pass_through_without_change();
    test_commit_then_pass();
    test_preedit_starts_composition();
    test_empty_preedit_ends_composition();
    test_cursor_clamped();
    if (failures) {
        std::fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    std::puts("edit plan: all tests passed");
    return 0;
}
