# Typing-habit benchmark

Pairs of Latin text and the Cyrillic word it should become. They measure how
well Buuz converts Mongolian that people type with their own Latin habits
(issue #42). The pairs are **synthetic**. They are not collected text.

| File | Content |
| --- | --- |
| `habits.tsv` | Generated pairs: `profile<TAB>latin<TAB>cyrillic` |
| `manual.tsv` | Hand-written pairs, same columns |

- **Source:** the 3,000 most frequent words of `data/wordlist/mn.txt`
  (Mongolian Wikipedia).
- **Profile:** one typing habit, for example `kh` (х typed as `kh`) or
  `u_for_ө`. Each profile changes one thing from the base spelling, which
  follows the table in `docs/typing-rules.md` section 3. The `mixed` profile
  combines habits that are common in chat.
- **Sample:** a profile only takes words that hold the changed letter. It
  takes every Nth word by rank, at most 300 words. The choice is
  deterministic.
- **Licence:** [CC BY-SA 4.0](LICENSE), like the word list it comes from.
  Attribution: Mongolian Wikipedia contributors.

## Generate and run

`core/tests/benchmark.rs` holds the generator and the report.

```sh
cd core
# Write habits.tsv again after a change to the generator.
BUUZ_BENCHMARK_WRITE=1 cargo test --test benchmark habits_file_is_current
# Print the score of every profile and the fix and break counts.
cargo test --release --test benchmark -- --ignored --nocapture --test-threads=1
```

The normal `cargo test` run checks that `habits.tsv` is current and that the
base profile stays above a score floor. Do not edit `habits.tsv` by hand.
Add hand-written pairs to `manual.tsv`.

The results and their limits are in
[docs/research/latin-habits.md](../../docs/research/latin-habits.md).
