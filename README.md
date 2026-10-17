# moult

**English noun inflection that checks its answers against a dictionary.**

```rust
assert_eq!(moult::singularize("waves"), "wave");
assert_eq!(moult::singularize("analysis"), "analysis");
assert_eq!(moult::pluralize("criterion"), "criteria");
```

Suffix rules alone get English plurals wrong in both directions. Some
over-fire ("waves" → "wafe", "police" → "polouse", "quota" → "quotum"); others
under-fire ("analysis" → "analysi", "gas" → "ga"). `moult` keeps the rules
but accepts a rule's output only when it is a noun
[Open English WordNet](https://en-word.net/) knows, and it reads irregular
forms ("mice", "criteria") from WordNet's own form list. A word WordNet does
not contain falls back to conservative rules.

- No dependencies, `no_std`.
- The word tables are compiled in: no runtime files, no WordNet install.
- Case is kept: "ISBNs" → "ISBN", "Territories" → "Territory".

**Status: skeleton.** The API is fixed; nothing is implemented yet. The
acceptance cases are in [`tests/cases.rs`](tests/cases.rs).

## Data

The tables in `src/data.rs` are generated from the Open English WordNet
release pinned in [`data/MANIFEST.toml`](data/MANIFEST.toml), by URL and
SHA-256, and committed. Moving to a later release is a deliberate change.

## License

The code is licensed under [Apache-2.0](LICENSE). The word tables are derived
from Open English WordNet and stay under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); attribution and the
Princeton WordNet 3.0 notice are in [LICENSE-DATA](LICENSE-DATA).
