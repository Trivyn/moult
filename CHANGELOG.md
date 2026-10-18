# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Every release entry has a `Host storage compatibility` subsection. It is also
copied into the annotated release-tag notes.

## [Unreleased]

### Added

- `S_REGULAR` in `src/policy.rs`: the closed list of nouns shaped like a
  plural that still take `-es` ("lens", "summons", "yes").

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
  ("dock," was "dock,s", "C++" was "C++s").

### Host storage compatibility

- Change: `pluralize` output changes for the 418 plural-shaped WordNet noun
  lemmas described under Fixed. Nearly all of the 0.1.0 outputs for them were
  non-words ("thankses"); the exception is "triceratops", whose
  "triceratopses" was a valid plural. The input is now returned unchanged.
  `cargo xtask report` lists the class ("plural-shaped noun lemmas left
  unchanged by pluralize").
- Change: `pluralize` output changes for a word that ends in a character that
  is not a letter or digit ("dock," to "dock,s" in 0.1.0). Those 0.1.0 outputs
  were non-words; the input is now returned unchanged.
- Affected seam: the canonical lexical forms returned by `pluralize` (and
  `Inflector::pluralize`). Host-visible output, not storage-internal.
  `singularize`, `is_noun` and the `guess_unknown` fallback are unchanged.
- Migration: none for hosts that persist singular forms. A host that persisted
  `pluralize` output from 0.1.0 should re-derive plurals for stored words, or
  map the old `-eses`/`-ses` non-words back to their input.

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
