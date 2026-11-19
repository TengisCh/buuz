// SPDX-License-Identifier: MIT OR Apache-2.0
#include "conf.h"

#include <vector>

namespace buuz {
namespace {

std::string trim(const std::string& s) {
    size_t a = s.find_first_not_of(" \t\r");
    if (a == std::string::npos) return std::string();
    size_t b = s.find_last_not_of(" \t\r");
    return s.substr(a, b - a + 1);
}

std::vector<std::string> split_lines(const std::string& text) {
    std::vector<std::string> lines;
    size_t pos = 0;
    while (pos < text.size()) {
        size_t nl = text.find('\n', pos);
        if (nl == std::string::npos) {
            lines.push_back(text.substr(pos));
            break;
        }
        lines.push_back(text.substr(pos, nl - pos));
        pos = nl + 1;
    }
    return lines;
}

std::string join_lines(const std::vector<std::string>& lines) {
    std::string out;
    for (const std::string& l : lines) {
        out += l;
        out += '\n';
    }
    return out;
}

// "[name]" gives "name"; any other line gives nullopt.
std::optional<std::string> header_of(const std::string& line) {
    std::string t = trim(line);
    if (t.size() >= 2 && t.front() == '[' && t.back() == ']') return trim(t.substr(1, t.size() - 2));
    return std::nullopt;
}

// "key = value # comment" gives key and value; other lines give false.
bool split_pair(const std::string& line, std::string* key, std::string* value) {
    std::string t = trim(line);
    if (t.empty() || t[0] == '#' || t[0] == '[') return false;
    size_t eq = t.find('=');
    if (eq == std::string::npos) return false;
    *key = trim(t.substr(0, eq));
    std::string v = trim(t.substr(eq + 1));
    if (!v.empty() && v[0] == '"') {
        size_t end = v.find('"', 1);
        *value = end == std::string::npos ? v.substr(1) : v.substr(1, end - 1);
    } else {
        size_t hash = v.find('#');
        *value = trim(hash == std::string::npos ? v : v.substr(0, hash));
    }
    if (key->size() >= 2 && key->front() == '"' && key->back() == '"')
        *key = key->substr(1, key->size() - 2);
    return true;
}

}  // namespace

std::optional<std::string> conf_get(const std::string& text, const std::string& section,
                                    const std::string& key) {
    std::string current;
    for (const std::string& line : split_lines(text)) {
        if (auto h = header_of(line)) {
            current = *h;
            continue;
        }
        std::string k, v;
        if (current == section && split_pair(line, &k, &v) && k == key) return v;
    }
    return std::nullopt;
}

std::string conf_set(const std::string& text, const std::string& section,
                     const std::string& key, const std::string& value) {
    std::vector<std::string> lines = split_lines(text);
    const std::string entry = key + " = " + value;

    std::string current;
    size_t section_end = lines.size();  // just after the last line of `section`
    bool in_section = section.empty();
    bool section_found = section.empty();
    for (size_t i = 0; i < lines.size(); ++i) {
        if (auto h = header_of(lines[i])) {
            if (in_section && !section.empty()) break;
            current = *h;
            in_section = current == section;
            if (in_section) {
                section_found = true;
                section_end = i + 1;
            }
            continue;
        }
        if (!in_section) continue;
        std::string k, v;
        if (split_pair(lines[i], &k, &v) && k == key) {
            lines[i] = entry;
            return join_lines(lines);
        }
        if (!trim(lines[i]).empty()) section_end = i + 1;
    }
    if (section.empty()) {
        // Top level: keep the keys above the first section header.
        size_t at = 0;
        for (size_t i = 0; i < lines.size(); ++i) {
            if (header_of(lines[i])) break;
            at = i + 1;
        }
        // Skip trailing blank lines before the first header.
        while (at > 0 && trim(lines[at - 1]).empty()) --at;
        lines.insert(lines.begin() + static_cast<std::ptrdiff_t>(at), entry);
        return join_lines(lines);
    }
    if (!section_found) {
        if (!lines.empty() && !trim(lines.back()).empty()) lines.push_back(std::string());
        lines.push_back("[" + section + "]");
        lines.push_back(entry);
        return join_lines(lines);
    }
    // `section_end` is just after the last non-blank line of the section.
    lines.insert(lines.begin() + static_cast<std::ptrdiff_t>(section_end), entry);
    return join_lines(lines);
}

std::string conf_with_version(const std::string& text) {
    for (const std::string& line : split_lines(text)) {
        std::string t = trim(line);
        if (t.empty() || t[0] == '#') continue;
        if (t.rfind("buuz_config", 0) == 0) return text;
        break;
    }
    return "buuz_config = 1\n" + text;
}

}  // namespace buuz
