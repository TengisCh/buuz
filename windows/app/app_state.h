// SPDX-License-Identifier: MIT OR Apache-2.0
// What the settings app remembers (language, theme, onboarding step) and
// how it edits typing options in settings.conf. No Windows headers.
#pragma once

#include <string>

#include "strings.h"

namespace buuz {

struct AppState {
    Lang lang = Lang::Mongolian;
    bool dark = false;       // light is the default, not the OS theme
    int step = 0;            // onboarding step to resume on
    bool onboarded = false;  // true after the last onboarding step
};

AppState parse_app_state(const std::string& text);
std::string write_app_state(const std::string& text, const AppState& state);

// The typing options the app shows as switches. `key` is the name in the
// `[options]` section. `soon` options are shown but cannot be changed yet.
struct TypingOption {
    const char* key;
    const char* label_id;
    bool default_on;
    bool soon;
};
const TypingOption* typing_options(size_t* count);

// Value of an option in the config text, or its default when it is missing.
bool option_on(const std::string& conf, const TypingOption& option);
std::string set_option(const std::string& conf, const TypingOption& option, bool on);

}  // namespace buuz
