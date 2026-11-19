// SPDX-License-Identifier: MIT OR Apache-2.0
#include "strings.h"

#include "strings_data.h"

#include <unordered_map>

namespace buuz {

namespace {

using Table = std::unordered_map<std::string, std::string>;

Table parse(const char* tsv) {
    Table t;
    const std::string text(tsv);
    size_t pos = 0;
    while (pos < text.size()) {
        size_t nl = text.find('\n', pos);
        std::string line = text.substr(pos, nl == std::string::npos ? std::string::npos : nl - pos);
        pos = nl == std::string::npos ? text.size() : nl + 1;
        if (!line.empty() && line.back() == '\r') line.pop_back();
        if (line.empty() || line[0] == '#') continue;
        size_t tab = line.find('\t');
        if (tab == std::string::npos) continue;
        t[line.substr(0, tab)] = line.substr(tab + 1);
    }
    return t;
}

}  // namespace

const std::string& tr(Lang lang, const std::string& id) {
    static const Table mn = parse(kStringsMn);
    static const Table en = parse(kStringsEn);
    static std::string missing;
    const Table& t = lang == Lang::Mongolian ? mn : en;
    auto it = t.find(id);
    if (it != t.end()) return it->second;
    // Fall back to English, then to the id.
    auto e = en.find(id);
    if (e != en.end()) return e->second;
    missing = id;
    return missing;
}

Lang other(Lang lang) { return lang == Lang::Mongolian ? Lang::English : Lang::Mongolian; }

}  // namespace buuz
