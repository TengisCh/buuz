/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/* C smoke test for buuz.h, run in CI against the static library. */
#include <stdio.h>
#include <string.h>

#include "buuz.h"

static int fail(const char *what) {
    fprintf(stderr, "smoke test failed: %s\n", what);
    return 1;
}

int main(void) {
    BuuzComposer *c = buuz_composer_new(NULL, 0, NULL);
    if (!c) return fail("new");
    const char *keys = "sain";
    BuuzOutcome out;
    for (const char *k = keys; *k; k++) {
        if (!buuz_process_key(c, BUUZ_KEY_CHAR, (uint32_t)*k, &out)) return fail("key");
    }
    size_t len = 0;
    const char *pre = buuz_preedit(c, &len);
    if (!pre || strcmp(pre, "сайн") != 0 || len != strlen("сайн")) return fail("preedit");
    if (buuz_cursor(c) != 4) return fail("cursor");
    if (!buuz_process_key(c, BUUZ_KEY_SPACE, ' ', &out)) return fail("space");
    if (!out.commit || strcmp(out.commit, "сайн") != 0) return fail("commit");
    if (out.consumed) return fail("space must pass through");
    if (buuz_is_composing(c)) return fail("composing");
    buuz_composer_free(c);
    if (strlen(buuz_version()) == 0) return fail("version");
    puts("buuz-ffi smoke test passed");
    return 0;
}
