# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Every release entry has a `Host storage compatibility` subsection. It is also
copied into the annotated release-tag notes.

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
