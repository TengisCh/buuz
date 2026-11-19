// SPDX-License-Identifier: MIT OR Apache-2.0
// Small text edits on a TOML-subset config file (docs/decisions/0005). No
// Windows headers, so tests run on any system. The edits keep comments,
// unknown keys and the order of lines.
#pragma once

#include <optional>
#include <string>

namespace buuz {

// Value of `key` in `section` ("" is the top level), as written, without
// quotes. Empty when the key is missing.
std::optional<std::string> conf_get(const std::string& text, const std::string& section,
                                    const std::string& key);

// Set `key = value` in `section`. `value` is written as given, so pass
// `true`, `3` or `"text"`. A missing key or section is added. Other lines
// stay as they are.
std::string conf_set(const std::string& text, const std::string& section,
                     const std::string& key, const std::string& value);

// `text` with `buuz_config = 1` as its first line, when it has none.
std::string conf_with_version(const std::string& text);

}  // namespace buuz
