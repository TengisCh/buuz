// SPDX-License-Identifier: MIT OR Apache-2.0
#include "edit_plan.h"

namespace buuz {

EditPlan plan(const Edit& edit, bool has_composition) {
    EditPlan p;
    // The core counts `delete_before` in Unicode scalars. We treat each scalar
    // as one UTF-16 unit. This holds because every Mongolian Cyrillic letter
    // is one UTF-16 unit. A character outside the BMP before the caret is
    // not supported.
    p.delete_utf16 = edit.delete_before;
    p.commit = edit.commit;

    // The same cases as the shell had before: a commit ends the composition,
    // and so does a key that leaves no composing word.
    p.end_composition = has_composition && (!edit.commit.empty() || !edit.composing);

    if (edit.composing && !edit.preedit.empty()) {
        p.preedit = edit.preedit;
        p.start_composition = !has_composition || p.end_composition;
        p.caret_utf16 = edit.cursor;
        if (p.caret_utf16 > p.preedit.size()) {
            p.caret_utf16 = static_cast<uint32_t>(p.preedit.size());
        }
    }

    p.touch_text = edit.consumed || !edit.commit.empty() || edit.delete_before != 0 ||
                   has_composition;
    return p;
}

}  // namespace buuz
