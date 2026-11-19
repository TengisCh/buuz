// SPDX-License-Identifier: MIT OR Apache-2.0
// UI text in Mongolian (default) and English. No Windows headers.
#pragma once

#include <string>

namespace buuz {

enum class Lang { Mongolian, English };

// UTF-8 text for `id`. An unknown id gives the id itself, so a missing
// string is easy to see.
const std::string& tr(Lang lang, const std::string& id);

// The language names a user picks from, and the other language.
Lang other(Lang lang);

}  // namespace buuz
