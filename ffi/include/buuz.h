/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/*
 * Buuz C ABI (buuz-ffi). Phonetic Latin to Mongolian Cyrillic for input
 * methods. The Rust source in ffi/src/lib.rs is the reference; keep this
 * header in step with it.
 *
 * Rules:
 * - Strings are UTF-8 with a byte length. Returned strings also end with a
 *   NUL byte.
 * - A returned string belongs to the handle and stays valid until the next
 *   call on that handle, or until the handle is freed.
 * - A null handle or a bad argument gives false, NULL or 0. A Rust panic
 *   never crosses this boundary.
 * - Use one composer from one thread at a time.
 */
#ifndef BUUZ_H
#define BUUZ_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct BuuzComposer BuuzComposer;
typedef struct BuuzWordList BuuzWordList;

/* Key kinds for buuz_process_key. */
enum {
    BUUZ_KEY_CHAR = 0,      /* a letter or input key; ch holds it */
    BUUZ_KEY_BACKSPACE = 1,
    BUUZ_KEY_DELETE = 2,
    BUUZ_KEY_LEFT = 3,
    BUUZ_KEY_RIGHT = 4,
    BUUZ_KEY_HOME = 5,
    BUUZ_KEY_END = 6,
    BUUZ_KEY_MODIFIER = 7,  /* Shift, Caps Lock: does not end the word */
    BUUZ_KEY_SPACE = 8,
    BUUZ_KEY_ESCAPE = 9,
    BUUZ_KEY_SYMBOL = 10,   /* printable, not a letter or space; ch holds it */
    BUUZ_KEY_TAB = 11,
    BUUZ_KEY_OTHER = 12     /* Enter and every other key */
};

/* Result of one key. Delete delete_before characters before the caret,
 * insert commit, then show the preedit. */
typedef struct BuuzOutcome {
    bool consumed;          /* false: pass the key on to the application */
    bool rejected;          /* the composition is full */
    bool corrected;         /* word choice or autocorrect changed the commit */
    uint32_t delete_before; /* in Unicode scalars */
    const char *commit;     /* UTF-8 or NULL; valid until the next call */
    size_t commit_len;      /* bytes, without the NUL */
} BuuzOutcome;

/* Word list in data/wordlist/mn.txt format. NULL on bad input. */
BuuzWordList *buuz_word_list_parse(const uint8_t *text, size_t len);
void buuz_word_list_free(BuuzWordList *words);

/* config: portable config text, or NULL for the smart defaults.
 * words: NULL, or a word list (the composer keeps its own reference).
 * NULL on a bad config. */
BuuzComposer *buuz_composer_new(const uint8_t *config, size_t len, const BuuzWordList *words);
void buuz_composer_free(BuuzComposer *c);

/* ch is a Unicode code point for BUUZ_KEY_CHAR and BUUZ_KEY_SYMBOL. */
bool buuz_process_key(BuuzComposer *c, uint32_t kind, uint32_t ch, BuuzOutcome *out);

const char *buuz_preedit(BuuzComposer *c, size_t *len);
uint32_t buuz_cursor(const BuuzComposer *c); /* in characters (= UTF-16 units) */
bool buuz_is_composing(const BuuzComposer *c);
const char *buuz_commit(BuuzComposer *c, size_t *len); /* NULL if empty */
void buuz_cancel(BuuzComposer *c);

const char *buuz_version(void);

#ifdef __cplusplus
}
#endif

#endif /* BUUZ_H */
