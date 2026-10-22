# moult: first real implementation, plan

> Plan mode only lets me write this file. **Step 0 after approval:** copy it to `tasks/todo.md`.
> Create `tasks/lessons.md` with the lesson from your P1 answer: *measure a class before
> curating it; let a default cover the bulk; refuse only the genuinely ambiguous rows*.

## Context

moult is a skeleton. The resolution order is documented in `src/lib.rs`, but the tables in
`src/data.rs` are empty, both functions `todo!()`, and the acceptance table in
`tests/cases.rs` is ignored.

This plan covers:
- a dev-only generator for pinned, byte-identical WordNet tables;
- the runtime, with an allocation-free fast path and a `Cow` return;
- a caller-supplied override layer;
- the policy decisions, with curated lists as Apache-2.0 source kept apart from the
  CC BY 4.0 tables;
- the checks that prove it works.

The first consumer is a deterministic NLP query pipeline calling on hot paths, so answers must
be safe to show users and calls must be cheap. The crate stays consumer-neutral.

**Decisions taken:**
- D1: packed noun index.
- P1: strip wins by default; refuse only the genuinely ambiguous classes; curated lists live in
  crate source, separate from the generated tables.
- P3: conservative unknown-word fallback.
- D4: `Cow<'_, str>` everywhere, including the free functions.

## What the data says (measured on the pinned release, sha256 checked)

| | count |
|---|---|
| Noun `LexicalEntry`s | 98,220 (97,618 distinct once lower-cased) |
| …containing a space (multi-word) | 49,002 → **dropped**: one word in can never match them, and no rule output from one word can be one |
| Single-token, written lower case | 44,543 → **`NOUNS`** |
| Single-token, written only capitalised (Ga, Kansas, Mr, Baptists) | 4,114 → **`NAMES`**: known words, never a rule target |
| Lemmas with `_` | 0 |
| Noun `<Form>`s | 1,391 of 4,473 |

The capitalised-only lemmas are what make lower-cased lookups go wrong: `gas`→`ga` (Ga,
gallium), `pus`→`pu`, `Kansas`→`Kansa`, `Mrs`→`Mr`. That is 48 collisions. Keeping
capitalised-only lemmas as known words but never as rule targets fixes all of them, and
Kansas is still left unchanged.

**Size of the ambiguity classes.** These are pre-measured with a script; step 1 re-measures
them with the generator's own rules:

| class | count | handling |
|---|---|---|
| A. input and its stripped form are both nouns (glasses/glass, arms, goods, metrics, physics) | ~460 | **default: strip wins**, the invariant list overrides it, no per-row list |
| B1. two lemmas share one regular plural (cookies: cookie/cooky, senses: sense/sens, corpses: corpse/corps) | 79 | **refused until classified** |
| B2. a WordNet form points at 2+ lemmas (leaves: leaf/leave, axes: axis/ax, bases, taxes) | 32 | **refused until classified** |
| C. an irregular form that is also a noun in its own right (men, teeth, data, stamina, cola, trivia) | 45 | **refused until classified** |
| D. pluralize: a lemma with 2+ WordNet forms, typos included (antheridium→`andtheridia`/antheridia, cry→`crying`) | 37 | **refused until classified** |

The refused total is about 193, which counts as "tens to low hundreds", so the go-ahead
condition holds. Step 1 is a **gate**: if the generator's count for B, C and D comes out in
the thousands, stop and pick a structural tie-break before writing any lists.

Other findings:
- WordNet's form list lacks `people`→person and `women`→woman. These go into a curated
  supplement.
- It decides `-oes`/`-ves` well: 83 of the 628 -o lemmas list an `-oes` form (potatoes,
  heroes), and 38 of the 269 -f/-fe lemmas list a `-ves` form (wolves, knives). It does not
  list photos or roofs.
- Some forms are only regular spelling variants (`busses`, `gasses`), or equal to their own
  lemma (`gas`, `genus`). Pluralize must not output these. The generator drops them from the
  lemma→plural table by a mechanical rule; see below.

**Size and compile cost** (macOS arm64, release, stripped, probe binary against an empty
343 KB baseline):

| noun index layout | source | binary cost | build |
|---|---|---|---|
| `&[&str]`, all 97.6k | 1.9 MB | +2.7 MB | 1.1 s |
| `&[&str]`, 48.6k | 788 KB | +1.2 MB | 0.6 s |
| **packed `&'static str` via `include_str!`, 48.6k (chosen)** | 456 KB | **+0.48 MB** | 0.2 s |

## Design

### Licence split: two kinds of table
- **Generated, CC BY 4.0** (`src/data.rs`, `src/nouns.txt`, `src/names.txt`): pure extraction
  from WordNet with mechanical transforms only. Every transform is listed in LICENSE-DATA's
  "Changes made": lower-casing, dropping multi-word lemmas, splitting out capitalised-only
  lemmas, dropping forms that are regular variants or equal to their lemma from the plural
  direction, and byte-order sorting.
- **Curated, Apache-2.0** (`src/policy.rs`, hand-written, a reason comment on every entry):
  - `INVARIANT`;
  - `IRREGULAR` supplement (people/person, women/woman, …);
  - `SINGULAR_CHOICE` (form → lemma, or "keep", for classes B1, B2 and C);
  - `PLURAL_CHOICE` (lemma → plural, for class D);
  - `EXCLUDED_FORMS` (typos, and non-plural forms like `crying`).

  `src/policy.rs` is ordinary crate source that the runtime consults *before* the generated
  tables. The generator never copies it into `data.rs`.

### Workspace and generator (dev-only, never shipped)
- The root `Cargo.toml` gets `[workspace] members = ["xtask"]`. `xtask/` has
  `publish = false` and the deps `quick-xml`, `flate2`, `sha2`, `ureq` and `toml`. Add the
  alias `.cargo/config.toml`: `xtask = "run -p xtask --"`.
  - The package's `include` list already leaves out `xtask/`, `data/` and `.cargo/`, so the
    runtime crate keeps zero dependencies.
  - Commit `Cargo.lock` (take it out of `.gitignore`) so generator builds are reproducible.
- The xtask compiles `src/rules.rs` and `src/policy.rs` in through `#[path]`. They are pure
  and have no data. This lets it simulate the runtime and validate the policy without a
  circular dependency or a hidden public API.
- `cargo xtask report` prints the counts for classes A–D and NAMES collisions, then every
  **unclassified** entry in B, C and D. Each entry comes as a ready-to-paste line with a
  suggestion: sense count first, then the spelling that only drops "s". It also lists
  **stale** policy entries that no longer match any ambiguity, and invariant or supplement
  entries that are not WordNet nouns.
- `cargo xtask gen-data`:
  1. reads `data/MANIFEST.toml`;
  2. downloads to `data/cache/` (gitignored) if missing;
  3. **refuses on a sha256 mismatch**;
  4. streams the gzip XML;
  5. **refuses if `report` would show unclassified, stale or invalid entries**;
  6. writes the generated files: byte-sorted, no timestamps, `\n` line endings, a header with
     source, edition, URL, sha256 and counts.

  `nouns.txt` and `names.txt` have no header so the binary search stays trivial. The
  `data.rs` header covers them and records each one's sha256. LICENSE-DATA names all three
  files.
- `cargo xtask gen-data --check` regenerates in memory and diffs against the committed files.
  It exits non-zero on any difference.
- `cargo xtask inflect` reads stdin and writes TSV, for the NLTK differential.
  `cargo xtask size` runs the cost measurement.

### Generated tables
- `NOUNS` and `NAMES`: `&'static str` from `include_str!`, one lower-case word per `\n`, sorted
  by bytes. `fn contains(table, word)` binary-searches over byte offsets and walks back to the
  previous `\n`. Nothing is parsed at startup and nothing is allocated.
- `FORM_TO_LEMMA: &[(&str, &str)]`, sorted by form, which allows several rows per form.
  `LEMMA_TO_PLURAL: &[(&str, &str)]`, sorted by lemma, with regular variants and self-forms
  filtered out. The generator guarantees that every key with more than one row is covered by
  `policy.rs`.

### Runtime
- **`src/case.rs`**
  - Lower-casing uses a 64-byte stack buffer when the word is ASCII. Longer or non-ASCII words
    fall back to an allocating `to_lowercase`.
  - Case is restored by copying the input's characters over the shared lower-cased prefix
    (ISBNs→ISBN, Territories→Territory).
  - Title Case input gives Title Case output (Mice→Mouse, Person→People).
  - For all-caps input, rewritten letters are upper case (MOUSE→MICE,
    TERRITORY→TERRITORIES). An appended suffix is lower case (ISBN→ISBNs, ID→IDs, BOX→BOXes),
    following the acronym convention.
- **`src/rules.rs`**
  - Checked singular rules, in this order:
    1. `-ies`→`-ie`
    2. `-ies`→`-y`
    3. `-(s|x|z|ch|sh)es`→strip `es`
    4. `-s`→strip
    5. `-oes`→`-o`
    6. `-ves`→`-f` / `-fe`

    The output must be at least 3 letters, so os→o and pas→pa never fire.
  - Regular plural: consonant+y→ies, sibilant→es, otherwise +s.
  - The unknown-word fallback (P3) is defined here too.
- **`src/lib.rs`** gives the resolution order. Update the crate docs to match.
  - **singularize:**
    0. overrides;
    1. acronym plural: an all-caps stem plus a lower-case `s`/`es` (IDs, SKUs, ETAs);
    2. `INVARIANT`, returned unchanged;
    3. `IRREGULAR` supplement, then `SINGULAR_CHOICE`, then `FORM_TO_LEMMA` minus
       `EXCLUDED_FORMS`;
    4. checked rules whose output is in `NOUNS`;
    5. known word (`NOUNS` ∪ `NAMES`), returned unchanged;
    6. fallback.
  - **pluralize:**
    0. overrides;
    1. `INVARIANT`, returned unchanged;
    2. already plural: the word is a known plural form but not a noun itself, or a checked rule
       maps it to a noun while it is not a noun. Returned unchanged, so cats stays cats.
    3. `IRREGULAR` supplement, then `PLURAL_CHOICE`, then `LEMMA_TO_PLURAL`;
    4. regular rules. -o and -f take plain -s unless WordNet lists `-oes`/`-ves`.
- **Public API**
  - `pub fn singularize(word: &str) -> Cow<'_, str>` and `pub fn pluralize(word: &str) -> Cow<'_, str>`
    return `Borrowed` when the word comes back unchanged. `Cow<str> == &str` still works in
    asserts.
  - `pub fn is_noun(word: &str) -> bool` is true if the word, or the singular that step 3 or
    step 4 gives, is a WordNet noun. It costs nothing extra and lets a host tell "unchanged
    because known" from "unchanged because unknown".
  - `Inflector` is the override layer: `Inflector::builder().pair("sku", "skus").invariant("personnel").build() -> Result<Inflector, OverrideError>`.
    - `build()` lower-cases and sorts once, into owned sorted views by singular and by plural.
      It rejects duplicate or conflicting pairs.
    - The result is immutable and `Send + Sync`, with no global state.
    - `inflector.singularize/pluralize(&str) -> Cow<'_, str>`; the free functions are the same
      code with an empty override set.
  - `OverrideError` implements `Display` and `Debug`. MSRV 1.70 rules out
    `core::error::Error`.

### Docs and release
- `README.md`: remove "skeleton", document the policies, overrides, `is_noun`, `Cow`, and the
  measured cost. Update the crate-doc example and the `tests/cases.rs` harness for `Cow`.
- `LICENSE-DATA`: rewrite "Changes made" to give the exact derivation, and say that curated
  policy lists are Apache-2.0 code, not WordNet data.
- `CHANGELOG.md`: new, with a 0.1.0 entry. Its **Host storage compatibility** subsection says
  that singularize/pluralize outputs (canonical lexical forms) are defined for the first time
  and are host-visible. Future WordNet or policy bumps change them and will be recorded per
  release. No migration is needed (0.0.0 panicked). Copy the subsection into the tag notes
  when tagging.
- The version bump and any commit happen only when you ask.

## Policy decisions
- **P1, settled as stated above.** Expected classifications, for review when the report
  prints them:
  - men→man, teeth→tooth, oxen→ox;
  - stamina, cola, trivia and hypochondria are kept;
  - data is kept (invariant);
  - leaves→leaf, axes→axis, bases→base, taxes→tax, masses→mass;
  - cookies→cookie, senses→sense, corpses→corpse;
  - antheridium→antheridia.

  Once the policy is stated, leaves and axes go into `tests/cases.rs`.
- **P2, invariant list. Recommendation; tell me if you want it changed.**
  - Hand-written in `src/policy.rs`, closed at about 80–120 entries. Each entry must be a
    WordNet noun, which the generator checks.
  - Not copied from any third-party inflector, so the licensing stays clean.
  - Seeds:
    - zero plurals: sheep, deer, fish, moose, swine, bison, salmon, aircraft, spacecraft,
      offspring;
    - -s singulars and mass-reading pluralia tantum: news, series, species, means,
      headquarters, logistics;
    - field-of-study -ics, taken one by one from the 31 found: physics, ethics, mathematics,
      economics and linguistics, but not metrics, graphics or tactics;
    - mass nouns whose -s form misleads in data text: data, metadata, information, equipment,
      software, feedback, research, evidence, personnel, police, cattle.
- **P3, settled: conservative fallback.**
  - It applies only to lower-case words of at least 4 letters, or to the acronym-plural shape.
  - Rules: `-ies`→`-y` after a consonant; sibilant+`es`→strip `es`; `-s`→strip after a
    consonant or after e, o or y.
  - Left unchanged: `-as`, `-is`, `-us`, `-ss`, capitalised words (Kubernetes, Athens,
    Mercedes), and words with digits or symbols.
  - Pluralize for unknown words uses the regular rules.
- **P4, pluralize for regular nouns. Recommendation.** `-oes`/`-ves` only where WordNet lists
  them (potatoes, heroes, wolves, knives); photos, roofs and chiefs get plain -s. The
  precedence is invariant, then irregular, then rules. Overrides beat everything.

## Implementation steps
1. **Gate.** Build the workspace and the xtask: manifest, download, sha256 refusal, XML
   streaming, `rules.rs`, `report`. Print the class counts. Stop and come back to you if B, C
   and D total in the thousands.
2. `src/policy.rs`: write the invariant list and the supplement, and classify every
   unclassified line the report prints until it is clean.
3. Writer: `data.rs`, `nouns.txt` and `names.txt`, with the deterministic header and
   `--check`.
4. Runtime: `case.rs`, the resolution order, the `Cow` API, `is_noun`, `Inflector`, crate
   docs.
5. Tests:
   - un-ignore `tests/cases.rs`, move it to `Cow`, and add rows for leaves, axes, corpses,
     cookies, senses, gas, Kansas, IDs and BOX→BOXes;
   - new `tests/overrides.rs` for SKUs, ETAs and personnel, and for conflicting pairs being
     rejected;
   - full-table unit tests:
     - singularize of every lemma returns itself or another noun, never a non-word;
     - singularize(pluralize(L)) == L for every lemma outside invariant and class A;
     - no pluralize output is in `EXCLUDED_FORMS`;
     - the unchanged path returns `Cow::Borrowed`.
6. NLTK differential, cost measurement, CI, README, LICENSE-DATA, CHANGELOG.

## Verification
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` are all clean, with the acceptance table un-ignored and passing.
  Update CI to use `--workspace`.
- **Byte-identical.**
  - `cargo xtask gen-data --check` passes.
  - An xtask unit test runs a small hand-written WN-LMF fixture through the generator twice,
    and once with shuffled entries, and gets the same bytes every time.
  - A new CI job `data` caches `data/cache/` keyed by sha256 and runs `--check`.
- **NLTK differential** (`tools/nltk-diff/`, dev-only, run with
  `uv run --with nltk==<pinned>` on NLTK's WordNet 3.0):
  - Word list: the 48.6k lemmas, their moult plurals, the 1,391 forms, and
    `/usr/share/dict/words` (235k).
  - Compare lower-case `singularize` with `WordNetLemmatizer().lemmatize(w, "n")`.
  - Every difference goes into a bucket: data version (PWN 3.0 vs OEWN 2025), invariant,
    class A/B/C choice, minimum stem length, NAMES, fallback (NLTK leaves unknown words
    unchanged), or bug.
  - Fix each bug and add a row for it in `cases.rs`.
  - Commit `DIFFERENCES.md` (counts plus each intentional class with examples) and
    `expected.tsv` (every intentional difference). A re-run must reproduce `expected.tsv`.
- **Package.**
  - `cargo package --list` must show exactly: `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`
    if cargo adds it, `README.md`, `LICENSE`, `LICENSE-DATA`, `src/*.rs`, `src/nouns.txt`,
    `src/names.txt` and `.cargo_vcs_info.json`.
  - A CI step diffs this list against `xtask/expected-package-files.txt`.
  - Record the `.crate` size; it should be about 200–300 KB, well under 10 MB.
- **Cost.** `cargo xtask size` reports the release binary delta of a probe built with and
  without moult, plus the clean build time of `-p moult` in debug and release. Record the
  results in the README. Targets: ≤ 0.6 MB and ≤ 2 s.
- **Hot path** (for information): ns/call over 100k mixed tokens, and the fraction of calls
  that return `Borrowed`.
- Compared with main, which panics, the tests above cover the behaviour difference.

## Notes and flags
- MOOSEDev MCP timed out this session, so no graph recall was possible. Once it reconnects:
  list-all recall, record D1, D4 and P1–P4 as `ArchitecturalDecision`s (P1's
  "measure before curating" as a `Lesson`), then validate.
- The API change to `Cow` overrides the "API is fixed" line in the brief, by your decision.
  The crate docs and README examples will be updated to match.
- `CLAUDE.md` §7 (SQLite, vector persistence) reads like it came from a downstream project.
  The CHANGELOG rule still makes sense here, because moult's outputs are canonical lexical
  forms. `CLAUDE.md` also has two "Development practices" sections. I'll leave it alone
  unless you ask.

---

## Status (2026-10-17): implemented, reviewed, uncommitted

All steps done; codex (`gpt-6-sol`) review rounds 1–5 run (rounds 4–5 because round 3 still had a
showstopper). Round 5: 0 showstopper, 1 major (IRREGULAR plurals not checked to be real words) —
fixed: an IRREGULAR plural must be a WordNet word or a compound of a WordNet irregular form.

Deviations from the plan above, each recorded in MOOSEDev:
- P4: no mechanical filter in the generator; doubled forms (busses, quizzes) classified by hand.
- P5 (new): PLURAL_CHOICE may prefer the modern plural over an archaic WordNet form.
- P6 (new): -man→-men (MAN_REGULAR), -sis→-ses, and /k/ -ch → -chs (CH_REGULAR).
- P7 (new): pluralize leaves recognised plurals unchanged.
- P3: the unknown-word fallback is opt-in (`Inflector::builder().guess_unknown(true)`); the
  default never guesses. Added src/others.txt (WordNet non-noun lemmas ending in -s).
- D8: case restoration is per character with an explicit origin (rule suffix vs form).
- LEMMA_TO_PLURAL is a u16 index into FORM_TO_LEMMA; xtask's moult dependency is behind the
  `inflect` feature so the generator never depends on its own output.
- Size: +0.61 MB release binary delta (target was ≤ 0.6 MB); clean build 0.3 s; .crate 230 KB.

---

## 0.2.0: GitHub issues #1 and #2 (ratified 2026-10-17)

MOOSEDev: P8 (2a19ead0…), D9 (5f94288a…), Constraint 8a614569… (supersedes 2b52d3f5…).

### Issue 1: plural-shaped noun lemmas (P8)
- [x] `rules.rs`: shared shape predicate: ends in `s` after anything but a/i/o/u/y, not `ss`,
      not a `-sis` word.
- [x] `policy.rs`: `S_REGULAR` = lens, summons, yes (sorted, reason comments).
- [x] `resolve::reads_as_plural`: a `Sing::Keep` noun of that shape, with no WordNet plural form,
      no IRREGULAR/PLURAL_CHOICE entry and not on `S_REGULAR`, is already plural.
- [x] xtask: validate `S_REGULAR` (lower-case noun, plural shape, not stale); `report` prints the
      class size.
- [x] Tests: `cases.rs` rows (thanks, clothes, earnings, geophysics, 1990s; lens→lenses,
      summons→summonses, yes→yeses, glans→glandes); full-table test in `table_tests.rs`.
- [x] Crate docs (pluralize step 3), NLTK `--check` still passes or expected.tsv re-derived.

### Issue 2: phrase entry points (D9)
- [x] `singularize_phrase` / `pluralize_phrase`: free functions and `Inflector` methods; inflect
      the last non-whitespace run, keep the rest byte-for-byte, Borrowed iff unchanged.
- [x] Docs: whitespace input unchanged on the word functions and `is_noun`; capitalised words
      never guessed (guess_unknown); README section; crate docs' closing paragraph.
- [x] `tests/phrase.rs`.
- [x] CLAUDE.md invariant 6 matches the new Constraint.

### Release
- [x] Version 0.2.0, CHANGELOG with Host storage compatibility (pluralize changes for 418
      lemmas), codex review rounds 1–3 (no showstopper left).
- [x] Commit, tag, push and publish only when asked (published 0.2.0, 2026-10-17).

Review fixes: round 1: pluralize leaves a word ending in a non-alphanumeric character
unchanged ("dock," was "dock,s"); S_REGULAR vs SINGULAR_CHOICE conflict rejected; report
membership follows the full singular order; docs say "nearly all" old outputs were non-words.
Round 2: S_REGULAR entries singularize reduces are rejected. Round 3: collins (Tom Collins)
stays unchanged instead of collinses: declined as within P8's accepted trade-off.

## 0.3.0: lemmatization by part of speech (ratified 2026-10-22)

Spec and decisions in MOOSEDev: Requirement "Spec: lemmatization by part of speech (0.3)",
D10 (candidates API), P9 (rules, order, LEMMA_CHOICE), D11 (WordNet only; overrides extend).
Constraint rewordings for invariants 1 and 6 proposed as supersessions (awaiting ratification).

### A. Generator and gate
- [x] xtask extracts single-word lower-case verb, adjective (+ satellite) and adverb lemmas and
      their listed forms; capitalised-only lemmas handled as for nouns.
- [x] Lemma rules (P9) in `src/rules.rs`, shared with the xtask.
- [x] `report`: per part of speech, lemma and form counts, max candidates, refused classes
      (listed/self involved) and rule-vs-rule counts. **Gate:** stop if refused is in the thousands.
- [x] `LEMMA_CHOICE` in `src/policy.rs` with reason comments; stale/invalid rows refused;
      exclusions extended with a part of speech.
- [x] Writes `src/verbs.txt`, `src/adjectives.txt`, `src/adverbs.txt` and per-POS form tables in
      `data.rs`; `gen-data --check` byte-identical.
      Gate result: L1 130 (79 verb, 47 adjective, 4 adverb), all classified; L2 314 (reported);
      max candidates 7 (better). Verb stems may be 2 letters (going→go); spelling-variant
      rows follow WordNet's listed lemma.

### B. Runtime
- [x] `src/lemma.rs`: `PartOfSpeech`, `Lemma`, `Lemmas` (fixed capacity, no allocation),
      `lemmas`, `lemmas_for`, `lemmatize`; Inflector methods; builder `.lemma(form, pos, lemma)`.
- [x] Noun candidates: singularize's checked answer first; `lemmatize(w, Noun) == singularize(w)`.

### C. Tests, docs, release
- [x] `tests/cases.rs` rows, `tests/lemma.rs`, full-table tests.
- [x] NLTK differential for pos v/a; SCOWL v2 coverage re-measured (dev-only, not committed).
      NLTK: 99.49% agreement over 322k words, 3,315 differences all bucketed, 0 bug.
      SCOWL v2 (size ≤ 70): verbs 98.5% correct (98.8% among candidates), adjectives 96.9%;
      most misses are other valid spellings (enrolled→enroll vs enrol).
- [x] `cargo xtask size`; README, crate docs, LICENSE-DATA, CLAUDE.md invariants 1 and 6,
      expected-package-files, CHANGELOG 0.3.0 with Host storage compatibility, version 0.3.0.
- [x] codex review rounds 1–3. Round 1 (after Phase B): 5 showstoppers fixed (policy keep
      words listed as lemmas, one-letter lemmas suppressed, capitalised-only V/A/R dropped —
      now known but never a rule target per D11, non-noun self override rejected, duplicate
      noun lemma overrides coalesced) and 2 minor (LICENSE-DATA, allocation wording).
      Round 2: 1 showstopper (OTHERS-only words offered rule nouns resolve rejects: alas→ala;
      gate now shared with resolve and mirrored in the generator) and 2 minor (e-restoring
      rules need a 2-letter stem, so bing/bed no longer reach be; nltk --check covers the
      lemma TSV, buckets require evidence). Round 3: no showstopper; 2 doc wordings fixed.
- [x] Commit and PR (2026-10-22). Tag and publish only when asked, after merge.
- Size (D12, user decision): no cargo feature. Noun-only binary unchanged (0.61 MB); +0.50 MB
  when lemmatize/lemmas are used; .crate 0.38 MB.
