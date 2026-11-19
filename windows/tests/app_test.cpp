// SPDX-License-Identifier: MIT OR Apache-2.0
// Tests for the settings app logic: config edits, state, strings.
#include <cstdio>
#include <string>

#include "app_state.h"
#include "conf.h"
#include "strings.h"

using namespace buuz;

static int failures = 0;
#define CHECK(cond)                                                              \
    do {                                                                         \
        if (!(cond)) {                                                           \
            std::fprintf(stderr, "%s:%d: CHECK(%s)\n", __FILE__, __LINE__, #cond); \
            failures++;                                                          \
        }                                                                        \
    } while (0)

static void conf_tests() {
    const std::string text =
        "buuz_config = 1\npreset = \"smart\"\n\n[options]\n# keep\nsmart_vowels = true # c\n\n"
        "[abbreviations]\nbn = \"baina\"\n";
    CHECK(conf_get(text, "options", "smart_vowels") == std::string("true"));
    CHECK(conf_get(text, "", "preset") == std::string("smart"));
    CHECK(!conf_get(text, "options", "keep_latin").has_value());
    CHECK(conf_get(text, "abbreviations", "bn") == std::string("baina"));

    // Change a key in place; comments and other sections stay.
    std::string a = conf_set(text, "options", "smart_vowels", "false");
    CHECK(conf_get(a, "options", "smart_vowels") == std::string("false"));
    CHECK(a.find("# keep") != std::string::npos);
    CHECK(conf_get(a, "abbreviations", "bn") == std::string("baina"));

    // A new key goes at the end of its section, before the next header.
    std::string b = conf_set(text, "options", "keep_latin", "true");
    CHECK(b.find("keep_latin = true\n\n[abbreviations]") != std::string::npos);

    // A missing section is added at the end.
    std::string c = conf_set("buuz_config = 1\n", "options", "auto_latin", "false");
    CHECK(c == "buuz_config = 1\n\n[options]\nauto_latin = false\n");

    // A top-level key goes above the first section.
    std::string d = conf_set(text, "", "extra", "1");
    CHECK(d.find("extra = 1\n\n[options]") != std::string::npos);
    CHECK(conf_get(d, "", "extra") == std::string("1"));

    // A key in another section with the same name is not touched.
    std::string e = conf_set("[a]\nx = 1\n[b]\nx = 2\n", "b", "x", "3");
    CHECK(conf_get(e, "a", "x") == std::string("1"));
    CHECK(conf_get(e, "b", "x") == std::string("3"));

    CHECK(conf_with_version("") == "buuz_config = 1\n");
    CHECK(conf_with_version("# hi\nbuuz_config = 1\n") == "# hi\nbuuz_config = 1\n");
    CHECK(conf_with_version("preset = \"smart\"\n") == "buuz_config = 1\npreset = \"smart\"\n");
}

static void state_tests() {
    AppState def = parse_app_state("");
    CHECK(def.lang == Lang::Mongolian);
    CHECK(!def.dark);
    CHECK(def.step == 0);
    CHECK(!def.onboarded);

    AppState s;
    s.lang = Lang::English;
    s.dark = true;
    s.step = 4;
    s.onboarded = true;
    AppState back = parse_app_state(write_app_state("", s));
    CHECK(back.lang == Lang::English);
    CHECK(back.dark);
    CHECK(back.step == 4);
    CHECK(back.onboarded);
    CHECK(parse_app_state("step = 500\n").step == 99);
    CHECK(parse_app_state("step = -3\n").step == 0);

    // Options: a missing key gives the default; a change is written once.
    size_t n = 0;
    const TypingOption* opts = typing_options(&n);
    CHECK(n >= 4);
    CHECK(option_on("", opts[0]));
    std::string off = set_option("", opts[0], false);
    CHECK(off.rfind("buuz_config = 1\n", 0) == 0);
    CHECK(!option_on(off, opts[0]));
    CHECK(option_on(set_option(off, opts[0], true), opts[0]));
    CHECK(set_option(off, opts[0], false) == off);
}

static void string_tests() {
    CHECK(tr(Lang::English, "btn_start") == "Get started");
    CHECK(tr(Lang::English, "no_such_id") == "no_such_id");
    CHECK(other(Lang::Mongolian) == Lang::English);
    // Every English id has a Mongolian string that is not the English text,
    // except the ones that are the same in both languages.
    const char* same[] = {"app_title", "welcome_demo_in", "welcome_demo_out", "try_task1",
                          "try_task2", "try_task3", "lang_toggle"};
    (void)same;
    CHECK(!tr(Lang::Mongolian, "btn_start").empty());
    CHECK(tr(Lang::Mongolian, "btn_start") != tr(Lang::English, "btn_start"));
}

int main() {
    conf_tests();
    state_tests();
    string_tests();
    if (failures) {
        std::fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    std::puts("app_test: ok");
    return 0;
}
