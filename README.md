# moult

**English noun inflection, verb third-person-singular forms and lemmatization checked against a dictionary.**

```rust
assert_eq!(moult::singularize("waves"), "wave");
assert_eq!(moult::singularize("analysis"), "analysis");
assert_eq!(moult::pluralize("criterion"), "criteria");
assert_eq!(moult::third_person_singular("live"), "lives");
assert_eq!(moult::lemmatize("made", moult::PartOfSpeech::Verb), "make");
```

Suffix rules alone get English plurals wrong in both directions. Some
over-fire ("waves" → "wafe", "police" → "polouse", "quota" → "quotum");
others under-fire ("analysis" → "analysi", "gas" → "ga"). `moult` keeps the
rules, but when singularizing accepts a rule's output only when it is a noun
that [Open English WordNet](https://en-word.net/) knows (plural rules form a
plural from the word itself, as a plural is not a lemma). Irregular forms
("mice", "criteria") come from WordNet's own form list. A word WordNet does not
contain comes back unchanged from `singularize`, unless you opt in to a
conservative guess (see [Unknown words](#unknown-words)).

All of them return a `Cow<'_, str>`, borrowed when the word comes back
unchanged, so the common case allocates nothing.

## How it decides

The resolution order is documented in the crate docs on
[docs.rs](https://docs.rs/moult) (`src/lib.rs`). For nouns, in short:

- Caller overrides come first. Nothing else can change their answer.
- Invariant nouns ("data", "news", "series") are unchanged.
- Irregular forms come from WordNet's form list, with curated choices for
  words that have more than one reading ("leaves" → "leaf").
- A word WordNet lists only under parts of speech other than noun (verbs,
  adjectives, adverbs; "alas" is filed as an adverb) is unchanged, unless a
  checked rule reduces it to a noun whose own plural is exactly that word
  ("bats" → "bat"; "alas" stays).
- Singular suffix rules apply only when their output is a WordNet noun.
- A word WordNet knows as singular is unchanged.
- A word no step above resolves is unchanged ("microservices" stays), unless
  the `Inflector` opts in with `guess_unknown(true)`.

Every answer is deterministic and, for singulars, checked against the dictionary;
guessing happens only if you opt in. The crate has no dependencies, is
`no_std` (with `alloc`), and compiles its word tables in, so there are no
runtime files and no WordNet install.

## Case

Lookup is case-insensitive, and output keeps the input's case pattern:

```rust
assert_eq!(moult::singularize("ISBNs"), "ISBN");
assert_eq!(moult::pluralize("ID"), "IDs");
assert_eq!(moult::singularize("Territories"), "Territory");
assert_eq!(moult::pluralize("MOUSE"), "MICE");
assert_eq!(moult::singularize("ISBN"), "ISBN");
```

Acronyms stay upper case. A noun rule suffix added to a whole all-caps word is
lower case ("ISBNs", "BOXes"); an irregular plural follows the word
("CHILD" → "CHILDREN"). All-caps verbs take an upper-case suffix too
("LIVE" → "LIVES", "DJ" → "DJS").

## Overrides

An `Inflector` adds caller-supplied pairs and invariant words. Its answers
take precedence over moult's own:

```rust
let inflector = moult::Inflector::builder()
    .pair("sku", "skus")
    .invariant("personnel")
    .build()
    .unwrap();
assert_eq!(inflector.pluralize("SKU"), "SKUs");
assert_eq!(inflector.singularize("personnel"), "personnel");
```

`third_person_singular(verb, form)` on the builder overrides a verb's present
form. It also registers the reverse verb lemma unless the two words are equal
(which keeps the verb unchanged):

```rust
let inflector = moult::Inflector::builder()
    .third_person_singular("can", "cans")
    .build()
    .unwrap();
assert_eq!(inflector.third_person_singular("Can"), "Cans");
assert_eq!(inflector.lemmatize("cans", moult::PartOfSpeech::Verb), "can");
```

`build()` returns `Err(OverrideError)` for duplicate or conflicting overrides,
including `DuplicateThirdPerson` for two overrides of the same verb. The
result is immutable and `Send + Sync`. The free functions behave like an
`Inflector` with no overrides.

## Unknown words

By default, a word that WordNet does not contain and that no dictionary-checked
rule reduces comes back unchanged from `singularize`. An acronym plural ("IDs",
"SKUs", "OSes") is still reduced, since that rule is structural. `pluralize`
and `is_noun` do not change with this option.

To have `singularize` and `third_person_singular` guess, opt in:

```rust
let inflector = moult::Inflector::builder()
    .guess_unknown(true)
    .build()
    .unwrap();
assert_eq!(inflector.singularize("microservices"), "microservice");
assert_eq!(inflector.third_person_singular("upsert"), "upserts");
// The free functions never guess.
assert_eq!(moult::singularize("microservices"), "microservices");
assert_eq!(moult::third_person_singular("upsert"), "upsert");
```

The noun guess takes a few conservative rules, and only when the shortened word is
a WordNet noun or ends in one of four letters or more after at least two more
letters; "laches" and "elaps" stay. It is best-effort for words WordNet lacks
and can produce a non-word for a compound whose last part is a noun
("afterhours" → "afterhour"). Leave it off when outputs are shown to users
without review.

The verb guess applies the regular verb rules only to a word of at least four
lower-case ASCII letters with no verb lemma candidates. It refuses endings
`s` (except `ss`), `ed` and `ing`; guessed `-o` words take plain `-s`.
Known verbs have the same answer with or without this option.

## Known words

`moult::is_noun` reports whether WordNet knows a word as a noun. It tells
"unchanged because the dictionary knows it" apart from "unchanged because
nothing does":

```rust
assert!(moult::is_noun("gas"));
assert!(!moult::is_noun("zorbit"));
```

## Phrases

`singularize`, `pluralize` and `third_person_singular` read one word: input
that contains whitespace is returned unchanged, and `is_noun` is `false` for it.
For a noun phrase, use
`singularize_phrase` and `pluralize_phrase`. They inflect the last word with
the matching word function (so overrides, case handling and `guess_unknown`
apply) and keep the rest of the text byte for byte:

```rust
assert_eq!(moult::singularize_phrase("order lines"), "order line");
assert_eq!(moult::singularize_phrase("Order Lines"), "Order Line");
assert_eq!(moult::pluralize_phrase("dry dock"), "dry docks");
assert_eq!(moult::pluralize_phrase("  dry dock\n"), "  dry docks\n");
```

Only the last word changes, so a head-initial compound inflects the wrong
word: "attorney general" gives "attorney generals", not "attorneys general".
Splitting an identifier ("OrderLines", "farm_workers") and inflecting a
compound's head is the caller's job.

## Third person singular

`third_person_singular` forms the third person singular present of a verb
lemma (the form used with "he", "she" or "it"). It checks the input against
WordNet's verbs, including verbs listed only with capitals, and comes back
unchanged for an unknown verb unless `guess_unknown(true)` is set. It is also
an `Inflector` method.

Most verbs take `-s`. Consonant + `y` becomes `-ies`; `s`, `x`, `z`, `ch` and
`sh` take `-es`. A closed list of consonant + `o` verbs takes `-es` ("go",
"do", "echo", "veto"); others take `-s` ("demo", "solo"). The listed
exceptions are "be" → "is", "have" → "has", and the doubled forms
"degas" → "degasses", "gas" → "gasses", "nonplus" → "nonplusses",
"quiz" → "quizzes" and "whiz" → "whizzes". Modals (can, could, may, might,
must, ought, shall, should, will, would) stay unchanged; need and dare are
ordinary verbs here.

```rust
assert_eq!(moult::third_person_singular("carry"), "carries");
assert_eq!(moult::third_person_singular("have"), "has");
assert_eq!(moult::third_person_singular("go"), "goes");
assert_eq!(moult::third_person_singular("can"), "can");
assert_eq!(moult::third_person_singular("LIVE"), "LIVES");
assert_eq!(moult::third_person_singular("DJ"), "DJS");
assert_eq!(moult::pluralize("BOX"), "BOXes");
```

For a verb phrase, `third_person_singular_phrase` (also an `Inflector` method)
inflects the FIRST whitespace-separated word, because verb phrases are
head-initial. It uses the same overrides, case handling and `guess_unknown`
setting, and keeps everything around that word byte for byte:

```rust
assert_eq!(moult::third_person_singular_phrase("live in"), "lives in");
assert_eq!(moult::third_person_singular_phrase("take place in"), "takes place in");
assert_eq!(moult::third_person_singular_phrase("  carry out\n"), "  carries out\n");
```

It does not find a verb later in a phrase: choosing the verb when the phrase
does not start with it is the caller's job. Words ending in punctuation stay
unchanged. Past tense and past participles ("make" → "made") are not generated.

## Lemmatization

`lemmatize` turns an inflected word into its dictionary form as a part of
speech you give it; `lemmas` and `lemmas_for` list the candidates:

```rust
use moult::PartOfSpeech::{Adjective, Verb};
assert_eq!(moult::lemmatize("made", Verb), "make");
assert_eq!(moult::lemmatize("Went", Verb), "Go");
assert_eq!(moult::lemmatize("better", Adjective), "good");

// Every reading of "saw": a noun, and two verb lemmas.
let found: Vec<_> = moult::lemmas("saw").map(|l| (l.text, l.pos)).collect();
assert_eq!(found.len(), 3);
```

- Candidates are checked against WordNet: each one is a lemma of the part of
  speech being produced, or an `Inflector` override. The input itself need not
  be listed ("hating" is not, yet gives "hate" and "hat"). A word for which no
  WordNet form or suffix rule reaches a lemma has no candidates, and
  `lemmatize` returns it unchanged. Lemmas are never guessed.
- `lemmas` takes no part of speech and lists nouns, then verbs, adjectives and
  adverbs. That order is not a preference between parts of speech; use
  `lemmas_for` or `lemmatize` when you know the part of speech. Within one part
  of speech the order is preferred-first: the override, the curated choice,
  WordNet's forms, the suffix rules, then the word itself.
- `lemmatize` keeps the input's case ("Went" → "Go", "MADE" → "MAKE"), but
  `Lemma::text` is the lower-case dictionary entry.
- For nouns, `lemmatize` is exactly `singularize`.
- A lemma WordNet writes only with capitals ("African") is known and lists
  itself, but no suffix rule ever reaches one.
- `Inflector::builder().lemma(form, pos, lemma)` overrides a lemma for one
  part of speech. A noun override acts as a singular/plural pair, so it also
  changes `pluralize` for that lemma.
- For the third person singular present ("make" → "makes"), use
  `third_person_singular`; past tense and past participles are not generated.

## Cost

Measured for 0.3.0 with `cargo xtask size` (macOS arm64, rustc 1.98.1; numbers
vary by target and toolchain). These measurements predate the verb generation API:

| Measure | Value |
|---|---|
| Release binary size delta, `singularize` + `pluralize` linked, stripped | +0.61 MB |
| Same, with `lemmatize` and `lemmas` also linked | +1.11 MB |
| Clean build time, `-p moult`, debug | 0.5 s |
| Clean build time, `-p moult`, release | 0.6 s |
| `.crate` size | 383 KB |
| Source size of `src/` | 1.1 MB |

The lemma tables are always compiled in. A program using only the noun functions
does not link the verb, adjective or adverb lemma tables: the noun-only cost is
unchanged from 0.2.0.

## Data

The tables in `src/data.rs` are generated from the Open English WordNet
2025 release pinned in [`data/MANIFEST.toml`](data/MANIFEST.toml), by URL and
SHA-256, and committed. The generator refuses to write the tables if the
download does not match the pinned hash. Moving to a later release is a
deliberate change, made separately from other work. `src/others.txt` lists the
words ending in `s` that WordNet lists only under parts of speech other than
noun, such as verbs, adjectives and adverbs ("perhaps", "besides", "alas"), so the opt-in unknown-word guess leaves them alone.

## License

The code is licensed under [Apache-2.0](LICENSE). That includes the curated
policy lists in `src/policy.rs`, which are hand-written decisions and not
WordNet data.

The word tables are derived from Open English WordNet and are licensed under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Attribution, the
exact derivation, and the Princeton WordNet 3.0 notice are in
[LICENSE-DATA](LICENSE-DATA).

## Development

The data generator and tooling live in the dev-only `xtask/` workspace member
and are never published. Commands run from the repository root:

```sh
cargo xtask report                  # policy counts; lists unclassified entries
cargo xtask gen-data                # regenerate src/data.rs from the pinned release
cargo xtask gen-data --check        # fail if the committed tables are stale
cargo xtask-inflect [--guess] < words.txt  # singularize/pluralize each line, as TSV (needs moult; see below)
cargo xtask-inflect --lemma < words.txt    # lemmatize each line, per part of speech (--lemma-candidates lists them)
cargo xtask size                    # binary and build-time cost
```

`cargo xtask-inflect` is an alias for `cargo run -p xtask --features inflect -- inflect`. The
`inflect` feature is the only path that compiles the moult crate into xtask, so the other commands
still build when `src/data.rs` is broken, and `gen-data` can repair it. Plain `cargo xtask inflect`
prints a pointer to the alias and exits non-zero.

CI runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo test --workspace` (which includes the acceptance cases in
`tests/cases.rs`). It also checks the packaged file list, builds and tests the
crate on Rust 1.70, and runs the data check.

The NLTK differential in `tools/nltk-diff/` compares moult with NLTK's
WordNet lemmatizer on Princeton WordNet 3.0 (`--lemma` adds the verb, adjective
and adverb comparison: 99.49% of word/part-of-speech pairs agree over 322k
words, and every difference is bucketed in `DIFFERENCES.md`). The noun
comparison runs in the default mode
(`expected.tsv`) and with `guess_unknown(true)` (`expected-guess.tsv`). Its
results are in `tools/nltk-diff/DIFFERENCES.md`. Its word list includes the
host's `/usr/share/dict/words`; the committed expected files record that file's
SHA-256, and `--check` refuses to run against a different one. It is run with:

```sh
uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py          # rewrite expected.tsv, expected-guess.tsv
uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --guess  # rewrite only expected-guess.tsv
uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --check  # fail if either would change
```

Changes are recorded in [CHANGELOG.md](CHANGELOG.md).
