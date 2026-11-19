// SPDX-License-Identifier: MIT OR Apache-2.0
// What to do in the text field after one key, in UTF-16 units. Pure logic,
// no Windows headers, so tests run on any system.
#pragma once

#include <cstdint>
#include <string>

#include "engine.h"

namespace buuz {

struct EditPlan {
    bool touch_text = false;  // false: no edit session is needed
    uint32_t delete_utf16 = 0;  // UTF-16 units to delete before the caret
    std::u16string commit;  // replaces the composition, or goes in at the caret
    std::u16string preedit;  // new underlined text, empty for none
    uint32_t caret_utf16 = 0;  // caret inside `preedit`, never past its end
    bool end_composition = false;  // end the current composition after the commit
    bool start_composition = false;  // start a new composition for `preedit`
};

// `has_composition` is true while the shell holds a TSF composition.
EditPlan plan(const Edit& edit, bool has_composition);

}  // namespace buuz
