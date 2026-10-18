# moult

**English noun inflection that checks its answers against a dictionary.**

```rust
assert_eq!(moult::singularize("waves"), "wave");
assert_eq!(moult::singularize("analysis"), "analysis");
assert_eq!(moult::pluralize("criterion"), "criteria");
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
[docs.rs](https://docs.rs/moult) (`src/lib.rs`). In short:

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

Acronyms stay upper case. A rule suffix added to a whole all-caps word is
lower case ("ISBNs", "BOXes"); an irregular plural follows the word
("CHILD" → "CHILDREN").

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

`build()` returns `Err(OverrideError)` for duplicate or conflicting pairs. The
result is immutable and `Send + Sync`. The free functions behave like an
`Inflector` with no overrides.

## Unknown words

By default, a word that WordNet does not contain and that no dictionary-checked
rule reduces comes back unchanged from `singularize`. An acronym plural ("IDs",
"SKUs", "OSes") is still reduced, since that rule is structural. `pluralize`
and `is_noun` do not change with this option.

To have `singularize` guess, opt in:

```rust
let inflector = moult::Inflector::builder()
    .guess_unknown(true)
    .build()
    .unwrap();
assert_eq!(inflector.singularize("microservices"), "microservice");
// The free functions never guess.
assert_eq!(moult::singularize("microservices"), "microservices");
```

The guess takes a few conservative rules, and only when the shortened word is
a WordNet noun or ends in one of four letters or more after at least two more
letters; "laches" and "elaps" stay. It is best-effort for words WordNet lacks
and can produce a non-word for a compound whose last part is a noun
("afterhours" → "afterhour"). Leave it off when outputs are shown to users
without review.

## Known words

`moult::is_noun` reports whether WordNet knows a word as a noun. It tells
"unchanged because the dictionary knows it" apart from "unchanged because
nothing does":

```rust
assert!(moult::is_noun("gas"));
assert!(!moult::is_noun("zorbit"));
```

## Phrases

`singularize` and `pluralize` read one word: input that contains whitespace is
returned unchanged, and `is_noun` is `false` for it. For a noun phrase, use
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

## Cost

Measured with `cargo xtask size` (macOS arm64, rustc 1.98.1; numbers vary by
target and toolchain):

| Measure | Value |
|---|---|
| Release binary size delta, `singularize` + `pluralize` linked, stripped | +0.61 MB |
| Clean build time, `-p moult`, debug | 0.3 s |
| Clean build time, `-p moult`, release | 0.3 s |
| `.crate` size | 230 KB |
| Source size of the tables (`nouns.txt`, `names.txt`, `others.txt`, `data.rs`) | 528 KB |

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
WordNet lemmatizer on Princeton WordNet 3.0, in the default mode
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
