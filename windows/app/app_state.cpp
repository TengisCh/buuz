// SPDX-License-Identifier: MIT OR Apache-2.0
#include "app_state.h"

#include <cstdlib>

#include "conf.h"

namespace buuz {

AppState parse_app_state(const std::string& text) {
    AppState s;
    if (auto v = conf_get(text, "", "lang")) s.lang = *v == "en" ? Lang::English : Lang::Mongolian;
    if (auto v = conf_get(text, "", "theme")) s.dark = *v == "dark";
    if (auto v = conf_get(text, "", "step")) {
        int n = std::atoi(v->c_str());
        s.step = n < 0 ? 0 : (n > 99 ? 99 : n);
    }
    if (auto v = conf_get(text, "", "onboarded")) s.onboarded = *v == "true";
    return s;
}

std::string write_app_state(const std::string& text, const AppState& s) {
    std::string out = conf_set(text, "", "lang", s.lang == Lang::English ? "\"en\"" : "\"mn\"");
    out = conf_set(out, "", "theme", s.dark ? "\"dark\"" : "\"light\"");
    out = conf_set(out, "", "step", std::to_string(s.step));
    out = conf_set(out, "", "onboarded", s.onboarded ? "true" : "false");
    return out;
}

const TypingOption* typing_options(size_t* count) {
    // The smart preset has these on, except autocorrect. `soon` rows need
    // the word list, which the Windows shell does not load yet.
    static const TypingOption options[] = {
        {"smart_vowels", "typing_smart_vowels", true, false},
        {"keep_latin", "typing_keep_latin", true, false},
        {"auto_latin", "typing_auto_latin", true, false},
        {"abbreviations", "typing_abbreviations", true, false},
        {"autocorrect", "typing_autocorrect", false, true},
        {"completion", "typing_completion", true, true},
    };
    *count = sizeof(options) / sizeof(options[0]);
    return options;
}

bool option_on(const std::string& conf, const TypingOption& option) {
    if (auto v = conf_get(conf, "options", option.key)) return *v == "true";
    return option.default_on;
}

std::string set_option(const std::string& conf, const TypingOption& option, bool on) {
    return conf_set(conf_with_version(conf), "options", option.key, on ? "true" : "false");
}

}  // namespace buuz
