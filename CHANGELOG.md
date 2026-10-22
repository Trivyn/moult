# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Every release entry has a `Host storage compatibility` subsection. It is also
copied into the annotated release-tag notes.

## [0.3.0] - 2026-10-22

### Added

- Lemmatization. `PartOfSpeech` (`Noun`, `Verb`, `Adjective`, `Adverb`), `Lemma`
  and `Lemmas`, and `lemmas`, `lemmas_for` and `lemmatize`, as free functions
  and as `Inflector` methods. Candidates are WordNet lemmas of the part of
  speech being produced (or overrides), never guesses, in a fixed
  preferred-first order within each part of speech: override, curated choice,
  WordNet forms, suffix rules, the word itself. `lemmas` lists nouns, then
  verbs, adjectives and adverbs, with no preference between parts of speech.
  `lemmatize` keeps the input's case; `Lemma::text` is the lower-case
  dictionary entry. For nouns `lemmatize` equals `singularize`.
- `InflectorBuilder::lemma(form, pos, lemma)` and `OverrideError::DuplicateLemma`.
  A noun lemma override acts as a singular/plural pair.
- Data: `src/verbs.txt`, `src/adjectives.txt`, `src/adverbs.txt` and the
  matching `verb_names.txt`, `adjective_names.txt` and `adverb_names.txt`, with
  `VERB_FORMS`, `ADJECTIVE_FORMS` and `ADVERB_FORMS` in `src/data.rs`.
  `LICENSE-DATA` describes the new derivation.
- `LEMMA_CHOICE` (129 curated rows for forms with more than one reading) and
  `LEMMA_EXCLUDED` in `src/policy.rs`.
- Dev tooling: `cargo xtask report` covers the lemma lists, `cargo xtask-inflect
  --lemma` and `--lemma-candidates`, and `tools/nltk-diff/diff.py --lemma`. The
  NLTK differential agrees on 99.49% of word/part-of-speech pairs over 322k
  words, with every difference bucketed in `tools/nltk-diff/DIFFERENCES.md`.
- `cargo xtask size` measures a lemma probe and reads the package version
  (it was hard-coded to 0.0.0).

### Changed

- The crate docs and README cover lemmatization, and the package description
  mentions it.
- A program that links the lemma functions grows by about 0.50 MB (+1.11 MB total
  against +0.61 MB for nouns only); one that uses only `singularize`,
  `pluralize` and `is_noun` is unchanged. The `.crate` grows from 0.24 to 0.38 MB.
- `singularize`, `pluralize` and `is_noun` output is unchanged (checked
  byte-identical over the full noun list).

### Host storage compatibility

None for existing data: `singularize`, `pluralize` and `is_noun` outputs are
unchanged, so no host migration is required.

New seam, host-visible: `lemmatize` and `lemmas` return new canonical lexical
forms. A host that persists lemmas should treat them as follows.

- The within-part-of-speech candidate order is canonical, because it decides
  what `lemmatize` returns. Future changes to `LEMMA_CHOICE`, the suffix rules or
  the pinned WordNet data change canonical forms and will be recorded here; a
  WordNet bump counts too.
- Candidate text (`Lemma::text`) is lower case, while `lemmatize` keeps the
  input's case pattern.
- A noun `.lemma()` override acts as a singular/plural pair, so it also changes
  `pluralize` for that lemma.

## [0.2.0] - 2026-10-17

### Added

- `singularize_phrase` and `pluralize_phrase`, as free functions and as
  `Inflector` methods. They inflect the last whitespace-separated word of a
  noun phrase with the matching word function (same overrides, case handling
  and `guess_unknown` setting) and keep the rest byte for byte ("Order Lines"
  → "Order Line", "dry dock" → "dry docks"). Head-initial compounds
  ("attorney general") and identifier splitting stay the caller's job.
  ([#2](https://github.com/Trivyn/moult/issues/2))
- `S_REGULAR` in `src/policy.rs`: the closed list of nouns shaped like a
  plural that still take `-es` ("lens", "summons", "yes").

### Changed

- The docs of `singularize`, `pluralize` and `is_noun` say that input
  containing whitespace is returned unchanged (`is_noun`: `false`), and the
  `guess_unknown` docs say that capitalised words are never guessed.

### Fixed

- `pluralize` no longer adds `-es` to a WordNet noun that is already plural.
  A noun that `singularize` keeps, that ends in `s` after a consonant, `e`, a
  digit or an apostrophe, and that has no listed plural, is now left
  unchanged: "thanks", "clothes", "earnings", "geophysics", "1990s" (were
  "thankses", "clotheses", …). Nearly all of these nouns are plural-only.
  418 plural-shaped WordNet lemmas are affected.
  ([#1](https://github.com/Trivyn/moult/issues/1))
- `pluralize` no longer appends a suffix after trailing punctuation: a word
  that ends in a character that is not a letter or digit is unchanged
  ("dock," was "dock,s", "C++" was "C++s"), and so is the last word of
  `pluralize_phrase("dry dock,")`.

### Host storage compatibility

- Change: `pluralize` output changes for the 418 plural-shaped WordNet noun
  lemmas described under Fixed. Nearly all of the 0.1.0 outputs for them were
  non-words ("thankses"); the exception is "triceratops", whose
  "triceratopses" was a valid plural. 0.2.0 returns the input unchanged.
  `cargo xtask report` lists the class ("plural-shaped noun lemmas left
  unchanged by pluralize").
- Change: `pluralize` output changes for a word that ends in a character that
  is not a letter or digit ("dock," to "dock,s" in 0.1.0). Those 0.1.0 outputs
  were non-words; 0.2.0 returns the input unchanged. Host-visible.
- Affected seam: the canonical lexical forms returned by `pluralize` (and
  `Inflector::pluralize`). Host-visible output, not storage-internal.
  `singularize`, `is_noun` and the `guess_unknown` fallback are unchanged.
- Migration: none for hosts that persist singular forms. A host that persisted
  `pluralize` output from 0.1.0 should re-derive plurals for stored words, or
  map the old `-eses`/`-ses` non-words back to their input.
- The new phrase functions add no new canonical forms: a phrase's result is
  its last word's word-function result, with the rest unchanged.

## [0.1.0] - 2026-10-17

### Added

- `singularize` and `pluralize`, returning `Cow<'_, str>`, borrowed when the
  word comes back unchanged.
- `is_noun`, which tells a word Open English WordNet knows as a noun from one
  it does not.
- `Inflector` and `InflectorBuilder`, for caller-supplied pairs and invariant
  words, with `OverrideError` for duplicate or conflicting pairs.
- `InflectorBuilder::guess_unknown(bool)`, an opt-in. By default a word that no
  dictionary-checked step resolves comes back unchanged from `singularize`;
  with the option on, a guarded fallback (known-head check, refusal endings)
  reduces regular plurals of words WordNet lacks ("microservices"). It is
  best-effort and can produce a non-word for a compound ("afterhours"). The
  acronym-plural rule, `pluralize` and `is_noun` do not depend on it.
- A dev-only generator (`cargo xtask gen-data`) that writes the word tables
  committed in `src/data.rs`, byte-identical for the pinned Open English
  WordNet 2025 release.
- Curated policy lists in `src/policy.rs`: invariant nouns, irregular forms,
  curated singular and plural choices, excluded forms, man-regular nouns and ch-regular nouns.
- `src/others.txt`: words ending in `s` that WordNet lists only under parts of
  speech other than noun (verbs, adjectives, adverbs). The opt-in unknown-word guess leaves them unchanged
  ("perhaps", "sometimes", "besides").

### Host storage compatibility

- Change: first release with defined outputs. 0.0.0 panicked on every call, so
  no host can have persisted moult output from it. No migration is required.
- Affected seam: the canonical lexical forms returned by `singularize` and
  `pluralize`. This is host-visible output, not storage-internal.
- Canonical forms of the default path (no `guess_unknown`) are what hosts
  should persist. Enabling `guess_unknown(true)` changes the canonical form of
  words outside WordNet ("microservices" becomes "microservice"), so a host
  must not switch the option on or off for stored data without a migration.
- Hosts that persist forms derived from moult should treat any future change
  to the pinned WordNet release, to `src/policy.rs`, or to the resolution rules
  as a change in canonical lexical forms. Later entries will record such
  changes here, with whether a host migration is needed.
