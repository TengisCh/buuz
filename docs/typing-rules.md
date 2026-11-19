# How Buuz turns Latin into Cyrillic

Status: describes `buuz-core` with smart vowel harmony (issue #7), which is
on by default, and abbreviation expansion (issue #5), which is opt-in; the original left-to-right behaviour is kept as the "classic"
option. Buuz is a port of the original by Odbayar Nyamtseren. Planned changes
are linked at the end and are **not** implemented yet.

Source: [`core/src/rules.rs`](../core/src/rules.rs) (the table) and
[`core/src/composer.rs`](../core/src/composer.rs) (the engine).

## 1. The big picture

1. While you type a word, Buuz keeps the **raw Latin keys** in a buffer and
   shows the converted word as underlined preedit text.
2. On **every key**, the whole word is converted again from the raw keys.
   This is why a later key can still change earlier letters (for example
   `'` after `o` turns о into ө).
3. A space, digit, punctuation mark, Enter, Tab or any other non-letter key
   **commits** the word and is then passed to the app unchanged.

Word characters are the ASCII letters `a`–`z`, `A`–`Z` and the two sign keys
`'` and `"`, plus the umlauts `ö`, `ü`, `Ö`, `Ü` (see below). Anything else
(including Cyrillic letters) ends the word. A word can hold at most 50 keys.

## 2. Matching: longest rule first

The engine reads the raw keys left to right. At each position it tries the
**longest** sequence in the table first (3 keys, then 2, then 1). So:

- `sh` → ш, not с + х
- `ch` → ч, `kh` → х
- `iii` → ы, `ii'` → ий, `ii` → ий or ы (see section 4), `i` → и
- `sxc` → щ

A key that matches nothing is copied as is.

## 3. The table

### Letters that never depend on the word

| Keys | Cyrillic | Keys | Cyrillic | Keys | Cyrillic |
| --- | --- | --- | --- | --- | --- |
| `a` | а | `j` | ж | `s` | с |
| `b` | б | `k` | к | `sh` | ш |
| `c` | ц | `kh` | х | `sxc` | щ |
| `ch` | ч | `l` | л | `t` | т |
| `d` | д | `m` | м | `v` | в |
| `e` | э | `n` | н | `w` | ү |
| `f` | ф | `p` | п | `x` | х |
| `g` | г | `q` | ө | `y` | ы |
| `h` | х | `r` | р | `z` | з |
| `i` | и | `ii` | ий or ы (smart, section 4) | `iii` | ы |
| `ya` | я | `ye` | е | `yo` | ё |
| `yu` | ю | `ii'` | ий | | |

A vowel followed by `i` becomes vowel + й: `ai` ай, `ei` эй, `oi` ой/өй,
`ui` уй/үй, `qi` өй, `wi` үй.

### Double V

With `Options::double_v` (config `[options] double_v`), the key pair `vv`
gives үү. A single `v` stays в.

| Typed | Result |
| --- | --- |
| `svv` | сүү |
| `VV` | ҮҮ |
| `Vv` | Үү |
| `avah` | авах (a single `v`) |

- The option is on in smart mode and off in classic mode. With it off, `vv`
  gives вв.
- `vv` counts as feminine for vowel harmony, like `w` (section 4). So
  `hvvhduud` gives хүүхдүүд.
- `vvi` gives үүй, like `wi` gives үй.
- `gvv` gives гүү and follows the same look-ahead as `gww` (section 4.1).
- A user key for `vv` in `[keys]` wins over this rule. A user key for `v`
  does not change `vv`: turn `double_v` off for that.
- The word list has few words with вв (`ввс`, `савва`). Type them with the
  option off, or in the Latin view.

### Habit keys

With `Options::habit_keys` (config `[options] habit_keys`), the key pair `ts`
gives ц and `zh` gives ж. Many people type these two pairs for the letters.

| Typed | Result |
| --- | --- |
| `tsagaan` | цагаан |
| `gezh` | гэж |
| `Tsagaan`, `TSAGAAN` | Цагаан, ЦАГААН |
| `ZH` | Ж |

- The option is on in smart mode and off in classic mode. With it off, `ts`
  gives тс and `zh` gives зх.
- ц and ж are consonants. They act like the keys `c` and `j` for vowel
  harmony.
- `ts` gives way when its `s` starts a longer key: `tsh` stays тш and
  `tsxc` stays тщ. Trade-off: `tssh` gives цш, because the second `s` starts
  `sh`, not the first. `zh` has no such exception: `dorzhhand` gives
  доржханд.
- A user key for `ts` or `zh` in `[keys]` wins over this rule.
- **With a word list.** Some words have тс or зх: `хүртсэн`, `тогтсон`,
  `багтсан`, `бүтсэн`. The composer checks the whole text so far. When it is
  not the start of a listed word, the composer tries тс (зх) at each habit
  key, then at both of two. It keeps the first result that starts a listed
  word. If none does, the result keeps ц and ж.
- The check uses the word list only, not the personal dictionary, like the
  ий check (section 4.2). A word with ий after a habit key is also tried with
  ий instead of ы: `batsuuriin` gives батсуурийн.
- The last word of a sentence can keep ц. When the ц spelling starts other
  listed words, it counts as a start of a word: `darts` gives дарц, not
  дартс.
- **The preedit can change while you type.** `bagts` shows `багц`, `bagtsa`
  shows `багца`, and `bagtsan` shows `багтсан`, because `багцан` starts no
  listed word. The word list is not a full spelling check: it decides only
  between these two spellings.
- **Without a word list** the keys always give ц and ж: `hwrtsen` gives
  `хүрцэн`. Turn the option off to type тс and зх, or type `t` and `s` in
  the Latin view.

### Vowels that follow vowel harmony

| Keys | Masculine word | Feminine word |
| --- | --- | --- |
| `o` | о | ө |
| `u` | у | ү |

To force one form regardless of the word:

| Keys | Result |
| --- | --- |
| `q`, `ö`, `o'`, `'o` | ө |
| `w`, `ü`, `u'`, `'u` | ү |
| `o"`, `"o` | о |
| `u"`, `"u` | у |

### Signs

| Keys | Result |
| --- | --- |
| `'` | ь |
| `''` | Ь |
| `"` | ъ |
| `""` | Ъ |

## 4. Vowel harmony

Every vowel rule carries flags: which kind of word it may appear in, and
whether it decides the word's gender.

- **Masculine:** `a`, `ya`, `yo`, `yu`, `o"`, `u"`.
- **Feminine:** `e`, `q`, `w`, `vv` (with `double_v`), `ö`, `ü`, `o'`, `u'` (and `ye`, see below).
- **Neutral:** `i`, `ii`, `iii`, `y`, all consonants and signs. Plain `o` and
  `u` do not decide anything; they follow the word.

"Smart vowels" is on by default. It can be switched off (the "Smart vowels"
checkbox on the website, `Options::classic()` in the core) to get the
original behaviour, described at the end of this section.

### 4.1 Look ahead for the gender

The whole word is converted again on every key, so the core first scans the
word for its **first deciding vowel** and starts in that gender. `o` and `u`
before it follow it. A word with no deciding vowel is masculine.

| Typed | Result |
| --- | --- |
| `unee` | үнээ |
| `huuhed` | хүүхэд |
| `ovqq` | өвөө |
| `hun` | хун (no deciding vowel: masculine, type `w` for хүн) |
| `ajlahgui` | ажлахгүй (the suffix -гүй, see 4.4) |

On screen, earlier letters can change when a later vowel arrives (`у` → `ү`
while typing `unee`). After the first deciding vowel the word still switches
gender left to right, as before.

Two keys are **not** evidence for the look-ahead (they still switch the
gender for the letters after them):

- `ye` (е): mostly a loanword vowel. `kofye` → кофе, `akadyemiig` →
  академийг.
- An explicit ү (`w`, `wi`, `u'`, `ü`, ...) right after `g`: it is the suffix
  -гүй. `bolohgwi` → болохгүй, `ugwi` → угүй.

### 4.2 ий or ы

`ii` becomes **ы** only when all of these hold; otherwise it is ий:

1. the word already has a vowel (`bii` → бий, `iim` → ийм, `chiig` → чийг);
2. the word is masculine at that point (`geriin` → гэрийн);
3. the letter before is a consonant other than ж ч ш щ ь ъ г к й
   (`bagshiig` → багшийг, `tuujiig` → туужийг, `achiin` → ачийн,
   `angiin` → ангийн, `airgiig` → айргийг);
4. the word so far is not one of the и-final stems салх, такс, компан
   (`salhiig` → салхийг).

A word list (`Options.words`) refines the result when rule 3 gives ы. If no
listed word starts with the text so far plus ы, and a listed word starts with
the text so far plus ий, the key pair gives ий. This covers stems that end in
ь: `surguuliin` → сургуулийн, `huuliin` → хуулийн, `huviin` → хувийн. Without
a list, `surguuliin` → сургуулын.

So `aaviin` → аавын, `nomiig` → номыг, `ohiniig` → охиныг, `uuliin` → уулын,
`hotiig` → хотыг. Capitals are kept: `AAVIIN` → ААВЫН, `Nomiig` → Номыг.

To override: `ii'` always gives ий (`aavii'n` → аавийн), and `y` or `iii`
always give ы. After г the rule keeps ий because back г (чарга → чаргыг) and
front г (айраг → айргийг) are typed the same.

### 4.3 The particle уу / үү

A word typed as just `u` or `uu` straight after a **space** takes the gender
of the word before it: `irsen uu` → ирсэн үү, `baina uu` → байна уу,
`sain uu` → сайн уу. Any other key in between (punctuation, Enter, a digit,
cancelling, or a caret move while nothing is being composed) forgets the
previous word, so `irsen. uu` → ирсэн. уу. Repeated spaces keep it, and so do
Shift and CapsLock (`Key::Modifier`). A caret move while a word is being
composed is consumed by the composer and does not forget it. Front ends send
the space as `Key::Space`; `Composer::commit()` (focus loss) and
`Composer::clear_context()` forget it on demand.

### 4.4 The negation suffix -гүй

The suffix -гүй is always гүй, also after a back vowel stem: ажлахгүй,
байхгүй, явахгүй, авахгүй, ажилгүй. In smart mode, with smart vowels on,
the key `ui` gives үй (not уй) when all of these hold:

1. the key before it gives г;
2. the text before that г has a vowel (`gui` → гуй, `guih` → гуйх,
   `gutal` → гутал);
3. the guard below does not keep гуй.

| Typed | Result |
| --- | --- |
| `ajlahgui` | ажлахгүй |
| `baihgui` | байхгүй |
| `chadaagui` | чадаагүй |
| `ajilgui` | ажилгүй |
| `ajlahguigeer` | ажлахгүйгээр |
| `ajilguichuud` | ажилгүйчүүд |

**Guard with a word list.** Take the text so far, which ends in г. Count the
best listed word that starts with the text plus уй, and the best one that
starts with the text plus үй. Keep гуй only when the уй count is greater.
`har'cangui` → харьцангуй (1188 against 21). `durgui` → дургүй (120
against 12). When neither spelling is listed, the rules without a list apply.

**Guard without a word list.** The text before г that ends in сан or сон
gives гүй (`baisangui` → байсангүй, `chadsangui` → чадсангүй). Otherwise,
keep гуй when the letter before г is н, or when the text before г is exactly
one of ду, шу, бу, а, огтор. So `dugui` →
дугуй, `shugui` → шугуй, `agui` → агуй, `ogtorgui` → огторгуй,
`har'cangui` → харьцангуй, `bolovsrongui` → боловсронгуй, `huraangui` →
хураангуй. The list is `GUI_STEMS` in `core/src/rules.rs`.

- **Override.** `u"i` always gives гуй (`ajlahgu"i` → ажлахгуй). `w` and
  `wi` always give гүй, as before.
- **Front suffix after it.** After the forced үй the word is feminine, so
  later `u` and `o` give ү and ө.
- **Not evidence.** The forced үй does not decide the gender for the look
  ahead, and not for the particle: `ajlahgui uu` → ажлахгүй уу.
- **Preedit.** The key `i` changes the text: `ajlahgu` shows ажлахгу, then
  `i` shows ажлахгүй.
- **Classic mode and smart vowels off.** No change: `ajlahgui` → ажлахгуй.
- **Ghost text.** With a word list, the ghost text for a preedit such as
  дургу can offer й, while the next `i` key gives дургүй.
- **Caret move.** After a caret move, the text is read back as Cyrillic.
  The rule does not apply to that text.
- **Custom keys.** The rule looks at what the keys give, so a key that
  gives г works the same.

### 4.5 Classic behaviour

With smart vowels off, the engine walks the word left to right starting in
masculine mode; each gender-deciding vowel switches the mode for the letters
after it. `unee` → унээ, `ovqq` → овөө, `aaviin` → аавийн, `irsen uu` →
ирсэн уу.

This is part of **classic mode** (`Options::classic()`), which types exactly
like the original Buuz with every smart feature off: no smart vowels, no
abbreviations, and none of the features added later. **Smart mode** is the
default (`Options::smart()` turns every smart feature on); each feature has
its own switch inside it. On the web converter the "Mode" switch under
"How to type" picks one; the choice is remembered in the browser.

### 4.6 What harmony does not cover

- **Words with no deciding vowel** (`hun`, `ug`, `huuhdiin`): genuinely
  ambiguous without a word list.
- **Other particles** typed as separate words.
- **ий vs ы after г**, and stems that end in и beyond the short list above.
- **Suffix spelling** (dropped vowels, ь before a suffix: `har'iin` stays
  харьийн). The suffix -гүй is the exception (section 4.4).

Background and sources: [issue #7](https://github.com/TengisCh/buuz/issues/7).

## 5. Capital letters

Case follows the keys you press, letter by letter: `Sain` → Сайн,
`SAIN` → САЙН, `sAin` → сАйн. For multi-key letters the first key decides
(`Sh` and `SH` → Ш). `''` and `""` give the capital signs Ь and Ъ.

## 6. Editing inside a word

Backspace and Delete edit the **raw keys** and the word is converted again.
Once you move the caret with the arrow keys, Home or End, the word is frozen
as Cyrillic and further edits act on Cyrillic letters, as in the original.
Frozen Cyrillic letters still count as evidence for the gender of the word:
`у` and `о` (like `а`) are masculine evidence, `ү`, `ө` and `э` feminine.

## 7. Abbreviations

A short form expands to a whole word when you type a space or punctuation
after it: `bn` and a space gives `байна` and a space, and `bn.` gives `байна.`. The end of the text
does not expand, so `bn` alone stays `бн`; on the web converter (which
converts the whole text again on every change) deleting the space therefore
brings the abbreviation back. The built-in list is small and
conservative; the literal conversion of every key (`bn` → бн) is not a real
word, so an entry never hides a word typed in full. `ni` gives the
particle нь; `ни` is not in the word list (issue #45).

| Type | Get | Type | Get |
| --- | --- | --- | --- |
| `bn` | байна | `bnu` | байна уу |
| `bga`, `bgaa` | байгаа | `sbnu` | сайн байна уу |
| `bhgui`, `bhgwi` | байхгүй | `sn` | сайн |
| `ym` | юм | `zgr` | зүгээр |
| `mdku` | мэдэхгүй | `tgd` | тэгээд |
| `ni` | нь | | |

- **Case** follows the keys: `Bn` → Байна, `BN` → БАЙНА.
- **Particle:** the expansion hands its gender to a following `uu`
  (`mdku uu` → мэдэхгүй үү, `bn uu` → байна уу).
- **Undo:** Backspace straight after an expansion committed by a space
  deletes the expansion and the space and brings back the literal preedit
  (`бн`), which then stays literal. `KeyOutcome::delete_before` tells the
  front end how many characters to delete first. Any other key forgets the
  undo; after punctuation or Enter there is no undo yet. The undo is for
  the desktop keyboards; the website has no Backspace hook and needs none.
  Shift and CapsLock keep the undo armed. `commit()`, `reset()`,
  `cancel()` and `clear_context()` drop it, so shells must call
  `clear_context()` on any caret move or focus change the composer did not see.
- **Never expands:** a word whose caret was moved (it is frozen Cyrillic),
  the focus-loss `Composer::commit()`, and the end of the text in the
  whole-text converters (`convert_with`, the website).
- **Settings:** off in the core by default (`Options::abbreviations` is
  `None`, also for `Options::classic()`), on by default on the web. A table
  is plain data (`Abbreviations::parse` / `to_text`, one `bn = байна` per
  line, `#` comments); the core does no I/O.

Not done yet: a per-entry editor, suffix shortcuts, undo after punctuation.
Background: [issue #5](https://github.com/TengisCh/buuz/issues/5).

## 8. Keeping Latin and reopening a word

Chat is full of English, names and links. Two keys keep them in Latin
without switching the keyboard off. Both are smart-mode only
(`Options::keep_latin`, off in `Options::classic()`), and both are for the
desktop keyboards: the website converts the whole text, not keystrokes.

- **Esc while composing** shows the raw Latin keys: `ok` shows `ок`, Esc
  shows `ok`, and a space or punctuation commits `ok`. Esc again brings the
  Cyrillic back, so nothing typed is lost. Case stays as typed (`GitHub`).
  Typing on, Backspace and Delete keep the Latin view. A Latin word never
  expands as an abbreviation. It gives no evidence for the particle, so a
  following `uu` is `уу`, as after no word. The next word is Cyrillic again.
- **Esc with nothing composed** goes to the application as before (it still
  closes dialogs). Esc after the caret moved inside a word does nothing:
  the word is frozen as shown (Cyrillic or Latin) and has no raw keys left,
  so Esc leaves it alone.
- **Backspace right after a word and a space** deletes both and reopens the
  word in the preedit with its raw keys (`sain` and a space, then Backspace,
  gives `сайн` in the preedit, ready for Esc or more keys). A word committed
  in Latin reopens in Latin. The previous word's gender comes back with it,
  so `irsen uu` reopened is still `үү`. The abbreviation undo in section 7 is
  the same gesture. The same limits apply: only straight after a space,
  Shift and CapsLock keep it armed, and any other key, punctuation, Enter,
  `commit()`, `reset()`, `cancel()` or `clear_context()` drops it. A frozen
  word (caret moved, or accepted by Tab, section 12) reopens too, from its
  Cyrillic text, and stays frozen: it never expands and Esc does nothing. For shell authors: with `keep_latin`,
  Backspace after any word and a space is consumed, so a missing
  `clear_context()` call after a caret move now matters after every word, not
  only after abbreviations.

Links and emails turn Latin by themselves: see section 9. Not done yet:
words with digits, a list of common English words, and per-app memory.
Background: [issue #10](https://github.com/TengisCh/buuz/issues/10).

## 9. Keeping links and emails in Latin

Links and emails are never Mongolian, so smart mode keeps them Latin
without a key press (`Options::auto_latin`, on in smart mode, off in
`Options::classic()`, separate from `keep_latin`). The website converter
follows smart mode. There is no switch on the page.

**Run.** A run is everything typed since the last Space, Enter or Tab:
words and the symbols between them (`github.com`, `bat@gmail.com`). The
composer keeps the raw keys of the run and the text it committed for it. It
tracks at most 100 characters. A longer run is never changed. A caret move
does not start a new run: it stops the tracking (and any Latin state) until
the next Space, Enter or Tab. Backspace outside a word and Esc outside a word
do the same.

**Key::Symbol.** A printable character that is not a letter and not a space
(`.` `@` `:` `/` `-` `_`, digits). Without `auto_latin` it acts exactly like
`Key::Other`. Enter, Tab and other non-printing keys stay `Key::Other`.

**Live triggers.** The run turns Latin at the key that completes one of:

- `@` anywhere in the run (`bat@`).
- `://` (`https://`).
- `/` typed right after a run that is already a domain (`github.com/`), so
  the path that follows stays Latin.

`www.` is not a trigger, because `www.` is also the Mongolian `үүү.`.
`www.google.com` turns Latin as a domain, at the end of the run.

The composer consumes the trigger key. `delete_before` is the number of
characters already committed for the run, and `commit` is the raw run plus
the key. The word still in the preedit was not committed, so it is not
counted. Words typed after this, in the same run, start in the Latin view
(Esc shows the Cyrillic of the current word).

**Domains at the end of a run.** At Space, Enter, Tab or the end of the
text, a run that looks like `label.label...tld` turns Latin the same way.
Before the check, the composer trims trailing `. , ; : ! ? ) ] " '` and
leading `( [ " '` from the run, so `(github.com),` and `visit github.com.`
work. The whole raw run is committed as typed, so the trimmed characters
stay as typed. Labels are letters, digits or `-`. The last label is one of:
com, net, org, edu, gov, io, dev, app, info, mn, ru, biz, xyz. Short country
codes that are also Mongolian words (`us`, `ai`, `de`, `me`, `co`, `tv`...)
are not in the list, because prose without a space after a full stop
(`tsai.de`) must stay Cyrillic. So `github.com` stays Latin, and `bn.ok`,
`sain.baina` and `ok.me` stay Cyrillic. A sentence end (`bn. sain`) is not a
domain. `sn.mn` turns Latin, because `mn` is in the list.

**Undo.** Backspace right after a domain turned Latin at a Space deletes the
Latin run and the Space, and commits the Cyrillic that was shown before the
change (what `auto_latin` off would have made, with abbreviations expanded
and a last label viewed in Latin with Esc still in Latin). The preedit stays
empty. Enter and the end of the text arm no undo. A live trigger has no
special undo: use Esc on the current word, or Backspace.

A run that turned Latin gives no evidence for the particle: `uu` after
`github.dev` is `уу`.

**Not done.** A caret move inside a word stops the detection for that run.
Words that mix digits and letters (`mp3`, `4k`) are not handled: chat uses
forms like `2r`, so this needs care (open on issue #10). A list of common
English words needs [#11](https://github.com/TengisCh/buuz/issues/11).
Per-app memory is shell work.

For shell authors: send `Key::Symbol(ch)` for these characters. Apply a
`KeyOutcome` in this order: delete `delete_before` characters before the
caret, insert `commit`, then show the preedit.

## 10. Custom keys

A user can add, replace or remove keys. The `[keys]` section of a config
(see [ADR 0005](decisions/0005-portable-config-format.md)) lists them, and
`KeyMap::from_pairs` builds them in the core. They work in both presets.

```toml
[keys]
x = "ш"       # x gives ш, like sh
kh = "ү"      # kh gives the same as w
vv = "у/ү"    # vv follows the gender of the word, like u
q = ""        # q is removed and stays q
```

- **The target.** The value names a lower case Cyrillic output that the
  built-in table already gives. The new key copies the rules of a built-in
  key that gives it: the vowel harmony flags, the gender variants and the case
  rule. The user does not edit flags, so a profile cannot break vowel
  harmony. An upper case target (`Ь`) is an error: upper case follows the key
  you press.
- **Which key is copied.** The first built-in key, in table order, whose
  rules give exactly the target. For `ү` that is `w`. For `у` it is `u"`.
  A pair such as `у/ү` copies the key that gives both: `u`. The pair `о/ө`
  copies `o`.
- **The `i` companion.** A key and the key plus `i` go together: `w` and
  `wi` (үй). When the copied key has such a companion, the new key gets it
  too, so `v = "ү"` gives `vi` = үй, and `v = "у/ү"` gives `vi` = уй or үй.
  Replacing or removing a key also replaces or removes its companion, unless
  another entry names the companion. Other longer built-in keys that start
  with the key (such as `yo`) are not changed.
- **Double V.** A key for `v` does not change `vv` (үү, section 3). Turn
  `double_v` off to type вв, or map `vv` itself.
- **Replace and remove.** A key that already exists loses its old rules, in
  every case combination. An empty value removes the key. It then passes
  through as Latin.
- **Case.** A key in all lower case gives lower case, whatever the length of
  the target: `y = "ий"` gives `ий`. In an upper case key, each key controls
  the letter at its own position, as in the built-in table: `Y` gives `ИЙ`,
  `V` gives `Ү`. A target without case conversion (`ь`, `ъ`) never changes
  case, and its key works in every case combination.
- **Longest match.** A new key of two or three characters takes part in the
  longest match as usual: with `kh = "ү"`, `khaan` gives `үаан`.
- **Smart rules follow the output.** The ий to ы rule, the rule that `ye`
  (е) gives no evidence for the gender, and the -гүй rules look at what the
  key gives, not at the letters typed. So `y = "ий"` gets ы where `ii`
  would, and `ii = "х"` gives х with no swap. One limit: the particle rule
  for `u` and `uu` still reads the typed keys, so a key mapped to `у/ү` is
  not a particle.
- **Errors.** A key must be 1 to 3 input characters. A target that no
  built-in key gives is an error (`KeyError::UnknownTarget`; a config gives
  `ConfigError::UnknownTarget` with the line number). Every entry copies from
  the built-in table, so entries do not depend on each other. A later entry
  for the same key wins.
- **Cost.** `KeyMap` holds a full copy of the table, built once.
  `Config::options()` builds it, so call that once per config change. The
  composer looks keys up in it as in the built-in table, so a keystroke costs
  the same and does not allocate. Without keys, the composer uses the shared
  built-in table.

## 11. Calibration

Calibration learns how a user types. The user types a few Cyrillic sentences
in Latin with conversion off, and `calibrate` gives suggested `[keys]` entries
(issue #4). It runs once, offline, and is not on the typing path. Nothing is
stored. "Default" here means the built-in table, not the keys a config
already has.

- **Alignment.** Each pair is a Cyrillic sentence and what was typed. Both
  are folded to lower case, and marks that Unicode may write apart (й as и
  plus a breve, an emoji variation selector) are made equal. Each Cyrillic
  letter matches a typed chunk of 0 to 3 characters, found by dynamic
  programming. A chunk holds only input keys (and digits, see below). A match
  costs 0 when the built-in table gives that letter from that chunk; the
  smart rule `ii` for ы counts as built-in. Any other chunk costs about 1 per
  character. A letter with no chunk, a typed character with no letter, and a
  non-letter that was not typed (a missing `?`) cost 1 each. Non-letters
  (spaces, punctuation, emoji) otherwise match themselves. Long vowels (оо,
  үү) are pairs of letters. о and ө, and у and ү, are separate letters.
- **Unmatched pairs.** A pair is left out of the counts, and its index goes
  into `unmatched`, when more than half of its letters have no partner (the
  user typed something else), when it is longer than 500 characters on
  either side, or when it is past the 50th pair.
- **Suggestions.** For each letter, a chunk that the table does not give is
  proposed when it was used at least twice and more often than the built-in
  chunks for that letter. Two such chunks for one letter are both proposed
  (an inconsistent typist). A suggestion is marked `tie` when another
  proposed chunk for the same letter was used as often.
- **Displaces.** When the chunk gives another letter in the built-in table,
  the suggestion names it: `v` for ү has `displaces` в, so the UI can warn
  that `v` will stop giving в.
- **Conflicts.** If the user also types that chunk for the letter the table
  gives from it (`h` for ц, and `h` for х), the chunk is reported in
  `conflicts`, with both letters and counts, and not proposed.
- **One chunk, one letter.** When two letters want the same chunk, the more
  common one is proposed and the other is a conflict that names it. When the
  top counts are equal, neither is proposed and there is one conflict per
  pair of letters.
- **Pair targets.** When one chunk wins for both о and ө, or both у and ү, it
  is one suggestion with `pair` set, for the target `о/ө` or `у/ү`, so the key
  follows vowel harmony (`y = "у/ү"`).
- **Unsupported.** A chunk that `[keys]` cannot take (a target that no
  built-in key gives, such as й, or a digit as a key) is listed in
  `unsupported` and not proposed.
- **Swaps.** `v` for ү and `w` for в gives both suggestions. Every `[keys]`
  entry copies from the built-in table (section 10), so
  `Calibration::apply` can add them together and the order does not matter.
  `apply` changes the config only when every entry is accepted.
- **Skipped letters.** Letters of the alphabet that are not in the input (or
  only in unmatched pairs) are listed, so the result can say what was not
  tested.

## 12. Completion

Completion shows the rest of a likely word as ghost text after the preedit,
and Tab accepts it (issue #2). It reads the word list
(`data/wordlist/mn.txt`, issue #11) and never leaves the device.

- **Switch.** `Options::completion` (config `[options] completion`) is on in
  smart mode and off in classic. It works only when `Options::words` holds the
  list. `Config::options()` cannot load data, so the shell reads
  `Config::needs_word_list()` first, loads the list and sets `words`. Without
  a list, or in classic, Tab is exactly `Key::Other` and nothing changes.
- **When there is a ghost.** `Composer::completion()` gives the suffix when
  all of these hold: the preedit is Cyrillic (not the Esc Latin view, not
  re-seeded), the caret is at the end, the preedit has at least 3 letters
  (converted Cyrillic letters, not typed keys), and the best word that starts with the lower-cased preedit is longer than
  the preedit and clearly ahead: it is the only candidate, or its count is at
  least twice the second one's. With custom keys the ghost follows the
  converted preedit.
- **Case.** An all upper-case preedit gives an upper-case ghost. Otherwise
  the ghost is lower case (`Баярл` gives `алаа`).
- **Accept.** Tab, or Right with the caret at the end, turns the preedit into
  preedit plus suffix. It is frozen as Cyrillic, like after a caret move: it
  is re-seeded, never expands as an abbreviation, and the caret is at the end.
  The word stays in the preedit: a suffix typed after it still composes (the
  gender evidence of the word holds, `гэрлэх` plus `ii` gives `гэрлэхий`), and
  Space commits it, and Backspace right after that Space reopens it (section
  8). Auto Latin stops for the rest of the run after an accept, as after any
  caret move (section 9): `гоогл` accepted, then `.com` stays Cyrillic. A run
  that already turned Latin shows no ghost. The accepted word decides the
  gender it leaves for a following particle. Tab with no ghost finishes the word and passes through,
  like `Key::Other`. `wants_key(Key::Tab)` is true only with a ghost.
- **Dismiss.** Typing on recomputes the ghost. Esc keeps its meaning (the
  Latin view), and the Latin view has no ghost.
- **Cost.** After each key the composer lower-cases the preedit into a reused
  buffer and asks the list for the top two words, with no allocation. The
  `completion()` returns a `&str` into the composer's buffer, so it does not
  allocate; the FFI and wasm layers copy at their boundary. A release-only
  test holds the worst key plus query under 10 microseconds (the budget of
  `docs/architecture.md` section 4); CI runs debug.

- **Web converter.** The page converts the whole Latin box at once and has no
  key-by-key composer. For the word at the caret it asks the wasm wrapper.
  `complete_at(before_caret, after_caret, config)` runs the word through one
  core `Composer` with the loaded list. It gives the ghost text, the whole
  Cyrillic word and how many Latin units the word replaces, or nothing. The
  rules stay in the core.
- **Word at the caret.** The word is the run of input characters that ends at
  the caret. A Latin run right after Cyrillic, a digit or any other character
  starts a new word. With an input character right after the caret (the caret
  is inside a word) there is no completion.
- **List loading.** The page loads `mn.txt` once with `load_word_list`, from
  its own origin, as a separate asset (not in the JavaScript). It loads on the
  first focus or input in the Latin box while completion is on, and when the
  switch is turned on while the box has focus. It does not load on page
  load. If the load fails, completion stays off with no message and the page
  does not try again in that visit.
- **Hint and Tab.** The hint under the Latin box shows the word. It shows only
  while the box has focus, nothing is selected and the caret is at the end of
  a word. Tab accepts the word only while the hint shows. It puts the Cyrillic
  word into the Latin box: the converter copies Cyrillic through unchanged, so
  the output is the same. Esc hides the hint for the current word, so the next
  Tab moves focus; the hint comes back when the word changes. With no hint,
  Tab keeps its normal focus behaviour. Touch screens do not show the hint.

## 13. Autocorrect

Autocorrect fixes a misspelled word when Space or punctuation ends it
(issue #1, stage 2). It reads the word list (section 12) and never leaves the
device. Stages 1 and 2 are free (`docs/architecture.md` section 7).

- **Switch.** `Options::autocorrect` (config `[options] autocorrect`) is off in
  `Options::default()`, `smart()` and `classic()`. It works only when
  `Options::words` holds the list. `Config::needs_word_list()` is true when
  autocorrect is on. Without a list, nothing changes.
- **When it runs.** Once per word, in the step that commits the word at Space
  or punctuation. It never runs on a key inside the word, and never in
  `Composer::commit()` (focus loss). Only Space and punctuation run it. Enter,
  Tab and Esc commit the word as typed. This holds also when Auto Latin is
  off. Only Space arms the undo: punctuation corrects with no Backspace undo,
  like abbreviations.
- **Skipped words.** Autocorrect leaves a word as typed when any of these hold:
  - the Latin view (Esc) shows the word, or the word is in a run that turned
    Latin (section 9)
  - the word was re-seeded (a caret move or Tab completion)
  - the word has fewer than 3 letters, or more than 16
  - a letter after the first is upper case (this includes all upper case)
  - the word is in the list
  - an abbreviation expanded it (section 7)
  - the word is in the rejected set, or Backspace just undid its correction
- **Candidates.** The search makes at most 2 edits of the lower-cased word,
  and keeps the results that are in the list:
  - `о` and `ө`, `у` and `ү`, `э` and `е`, `ий` and `ы`
  - a single vowel and a double vowel: it adds or removes one copy of a
    vowel next to itself
  - it adds or removes `ь`
  - a final vowel is never edited: no `э` and `е` swap on the last letter,
    and no single or double vowel change at the end of the word (`идэе`,
    `хүлээе`, `захиаа` and `шинээ` stay as typed)
- **Harmony slips.** A word like `хуухэд` needs two edits of one kind: `у` to
  `ү`, twice. Two edits cover it. A word that needs 3 edits is not corrected.
  With smart vowels on, most harmony slips are already fixed while typing, so
  autocorrect mostly fixes double vowels, `ь` and slips the harmony rule
  cannot see.
- **Choice.** The winner has the highest count. It needs a count of at least
  50, and a count at least 3 times the next candidate's. A single candidate
  needs only the count of 50. Otherwise the word stays as typed.
- **Case.** A leading capital stays (`Хуухэд` gives `Хүүхэд`).
- **Undo.** Backspace right after the Space reopens the word with its raw
  keys (section 8) and shows the typed conversion. The word goes into the
  rejected set. Space then commits it as typed: a flag on the reopened word
  makes sure of this, also when the set is full. The set holds at most 256
  words for the session. When it is full, a new undo is not remembered, so
  that word can be corrected again later. The personal dictionary
  (section 14) remembers the undo across sessions.
- **Shells.** `KeyOutcome::corrected` is true when the commit holds a changed
  word, so a shell can flash or underline it.
- **Web.** The converter turns the whole Latin box into Cyrillic in one call
  (`convert_with_config`). With the switch on and the word list loaded, it
  corrects each word that a Space or punctuation ends, as the core does. A
  newline or the end of the text does not correct. The switch is in the
  settings dialog, off by default. The page loads the word list when the
  switch is on, then converts again. The converter has no Backspace undo.
  It shows the corrected text. To stop corrections, turn the switch off.
- **Cost.** The search runs once per word. It reuses its buffers and checks
  each edit with `WordList::count`, so it allocates only the result. A
  release-only test holds the worst word under 1 millisecond. A word of 16
  letters that fits many edits (`oeoeoeoeoeoeoeoe`) is the worst case: about
  0.4 to 0.8 milliseconds on the development machine. A usual word takes
  about 50 microseconds.

### 13.1 Word choice (issue #64)

Word choice is a smart feature that works like autocorrect, but it makes
fewer kinds of edit. It is on in smart mode and off in classic mode.

- **Switch.** `Options::word_choice` (config `[options] word_choice`). It needs
  `Options::words`. Without a list, nothing changes.
- **When it runs.** Once per word, at Space or punctuation, with the same skip
  rules as autocorrect. Backspace right after a Space undoes it, and the word
  is not changed again.
- **Candidates.** Each `о` may be `ө`. Each `у` or `ү` may be `у`, `ү` or
  `ө`. It checks every combination for at most 5 such letters. A word with
  more of them stays as typed.
- **Extra edits.** For a word with at most 3 such letters, each combination
  also gets one more edit: `э` to `е` (`wed` gives `үед`), a doubled vowel
  (`terer` gives `тэрээр`), `и` to `ь` (`huuli` gives `хууль`), or an added
  `ь` (`surguul` gives `сургууль`). No edit changes the last letter's vowel
  length, and `э` to `е` skips the last letter.
- **Choice.** All candidates share one tally. The winner has a count of at least 50, at least 3 times the
  next candidate's, and at least 50 times the count of the word as typed
  (0 if it is not listed). The last rule keeps `уурын` (642) against
  `өөрийн` (7898), and still changes `мон` (114) to `мөн` (20067).
- **Limit.** `уруу` (399) and `өрөө` (333) are both common. The list cannot
  choose, so `uruu` stays `уруу`.
- **Autocorrect.** When both are on, word choice runs first.

## 14. Personal dictionary

The personal dictionary (issue #12) holds the words Buuz learned from the
user. It stays on the device. The core has the `UserDict` type and the
learning events. It does no I/O: each shell keeps the text itself.

- **What it holds.** Three kinds of entry:
  - `add`: a word the user added. It is always valid and never corrected.
  - `reject`: a word whose correction the user undid. It is never corrected
    again, also in a later session.
  - `use`: a known word (in the word list or the dictionary) that the user
    committed, with a count.
- **Limits.** At most 5,000 entries. A word has 64 characters at most, and no
  white space. When the dictionary is full, a new entry evicts the least
  recently used `use` entry. It evicts an `add` or `reject` entry only when
  no `use` entry is left, so normal typing never removes a word the user
  added or a rejection. When only added and rejected entries are left, a new
  `use` count is dropped. Using an added word keeps it fresh. A logical
  counter orders the use, not the system time.
- **Text format.** One `kind<TAB>word<TAB>count` line per entry, oldest use
  first. `UserDict::parse` skips a bad line and reports it as a warning, so one
  bad line does not cost the whole file. It refuses a text over 1 MiB. A text
  with more than 5,000 entries keeps the last 5,000 (`use` entries go first).
  A line after the cap still gets a `Duplicate` warning. The count of an
  `add` line is always 1. A 1 MiB file parses in about 20 milliseconds in
  release. `UserDict::to_text` writes the format. The dictionary is not part of the shareable config
  (section 10 and ADR 0005), because it holds what the user typed.
- **Learning without a lock.** The composer reads a snapshot,
  `Options::user: Option<Arc<UserSnapshot>>`, built by `UserDict::snapshot()`,
  and never changes it. It returns events in `KeyOutcome::learned`:
  - `Learned::Used(word)` on the key that commits a word at Space or
    punctuation, when the word is in the word list or added to the dictionary.
    A word that Tab accepted counts too.
  - `Learned::Rejected(word)` on the Backspace that undoes a correction

  The words are lower case. A key that is not one of these has `None`, and
  allocates nothing for learning.
- **No learning.** The composer returns no event when `Options::learning` is
  false, in classic mode, in the Latin view, in a run that turned Latin, for a
  word that was re-seeded, for a word that Backspace reopened, for an
  abbreviation expansion, and for a corrected word. `Options::learning` is true
  in `default()` and `smart()`, and off by default in `classic()`. The config
  option `[options] learning` overrides it, so a config can turn it on in
  classic mode. Shells turn learning off for password fields and, where the
  platform tells them, for private windows.
- **Domains.** A known word before a `.` is learned when the `.` commits it,
  before the run turns Latin at the end (`sain.mn` gives `Used("сайн")`). A
  word before `@` gives no event, because the `@` turns the run Latin first.
- **Shells.** The shell applies each event with `UserDict::apply` to its own
  dictionary, and saves it later, not on the typing path. It builds a new
  snapshot with `UserDict::snapshot()` (a sort of at most 5,000 words) and
  gives it to the composer with `Composer::set_options` when a field gets
  focus or after an idle pause, not after every word. The new snapshot shows the
  change from then on. Autocorrect also skips the words rejected in the
  session (section 13), so an undo works at once.
- **Web.** The converter has no composer and no Backspace undo, and it
  converts the whole Latin box again on each input. So it never learns: it
  records no `use` and no `reject` entry, because the same word would count
  many times. It only reads the dictionary. The page keeps the text in
  `localStorage` under `buuz.userdict` and gives it to wasm with
  `set_user_dict`, which builds the snapshot again only when the text
  changes. Autocorrect (`convert_with_config`) and completion (`complete_at`)
  use that snapshot, so an added word is not corrected and Tab can finish it.
  The settings dialog lists the entries, adds a Cyrillic word (the core
  checks it), deletes one, clears all with a confirm, and exports or imports
  a separate `.txt` file. An import merges, keeps the entries already there
  and shows a warning for each skipped line. The file is not part of the
  config export. A learning switch would change nothing on the web, so there
  is none. Learning a use once per word, for example when Tab accepts a
  completion, is possible later.
- **Completion.** Completion (section 12) first asks the snapshot. A user
  word is an added word, or a word with 3 uses or more, and not a rejected
  word. The word as typed counts: if it has the most uses, no ghost shows.
  Otherwise the user word with the most uses wins, and the most recent one
  wins a tie. It beats every list word with the same prefix, and it needs no
  clear margin. The snapshot keeps the candidates sorted, finds the prefix
  range with a binary search and scans only that range, with no allocation.
  With 5,000 entries spread over many prefixes, a key takes about 1
  microsecond in release (the test holds it under 10 microseconds). If all
  5,000 words share one prefix, the scan covers all of them and takes about 25
  microseconds.
- **Autocorrect.** Autocorrect (section 13) never corrects an added or a
  rejected word. An added word is known, like a list word.
- **Rejection.** An undo of a correction stores the word as the user typed
  it. That word may not be in the word list: it can be a name.
- **Privacy.** The dictionary is in memory in the core and goes to the shell
  only as return values. The core makes no network call and writes no file.
  The user must be able to see, delete and clear it in the shell.

## 15. Cyrillic to Latin

Not implemented. The core only converts Latin to Cyrillic. A reverse
converter is on the roadmap for the web converter. If added, it should
produce keys that convert back to the same text with the default table
(`ү` → `w`, `ө` → `q`, `х` → `h`, `ц` → `c`, `ы` → `y`, `ь` → `'`), so that
round trips are exact.

## 16. Planned changes

- [#8](https://github.com/TengisCh/buuz/issues/8) portable config and
  smarter defaults
- [#13](https://github.com/TengisCh/buuz/issues/13) the full smart typing plan
