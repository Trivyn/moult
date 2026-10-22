# moult

English noun inflection and lemmatization (`singularize`, `pluralize`, `lemmatize`) checked against Open English WordNet. A
suffix rule's output is accepted only when it is a WordNet noun; irregular forms come from
WordNet's form list; unchecked rules apply only to words WordNet does not contain. The crate
docs in `src/lib.rs` define the resolution order — keep them and the code in step.

## Invariants

1. **Deterministic, dictionary-checked.** No heuristic guesses without a WordNet check against
   the part of speech being produced (noun for singularize/pluralize/is_noun; the requested
   part of speech for lemmas/lemmas_for/lemmatize). The only exception is the noun fallback for
   words WordNet does not contain (opt-in `guess_unknown`), and it stays conservative: leaving
   a word unchanged beats inventing a non-word. Lemma candidates never come from guessing.
2. **No dependencies, `no_std` + `alloc`.** The runtime crate pulls in nothing. Tooling (the
   data generator) lives outside the published crate and may use what it needs.
3. **Data is pinned and committed.** `src/data.rs` is generated from the release pinned in
   `data/MANIFEST.toml` (URL + SHA-256) and committed, so a build needs no network. Never
   hand-edit generated tables; change the generator or the policy lists instead. Bumping the
   WordNet release is a deliberate, separate change.
4. **Licences travel with the data.** Code is Apache-2.0; the word tables are CC BY 4.0
   (Open English WordNet) with the Princeton WordNet 3.0 notice. `LICENSE-DATA` must ship in
   the package (`include` in `Cargo.toml`) and must describe any change in how the data is
   derived.
5. **Case is kept.** Lookup is case-insensitive; output keeps the input's case pattern
   (acronyms stay upper case, capitalised words stay capitalised, "Made" gives "Make").
   The one exception: lemma candidates (`Lemma::text`) are lower-case dictionary keys.
6. **One word in, one word out.** `singularize`, `pluralize`, `is_noun`, `lemmas`,
   `lemmas_for` and `lemmatize` take one word; input containing whitespace is returned
   unchanged (`is_noun`: false; `lemmas`: empty). `singularize_phrase` and
   `pluralize_phrase` inflect only the final whitespace-separated word and keep the rest
   byte-for-byte. Splitting identifiers (camelCase, snake_case) and choosing the head of a
   head-initial compound ("attorney general") stay the caller's job.
7. **Consumer-neutral.** This is a public crate: no references to specific downstream
   projects in code, docs, or tests.

## Development practices

### 1.  Plan Mode Default
- Enter plan mode for ANY non-trivial task (#+ steps or architectural decisions)
- If something goes wrong, STOP and re-plan immediately - don't keep pressing on.
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One task per subagent for focused execution

### 3. Self improvement Loop
- After ANY correction from the user: update `tasks/lessons.md` with the Pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

### 4. Verification before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution."
- Skip this for simple, obvious fixes -- don't over-engineer
- Challenge your own work before presenting it.

### 6. Autonomous Bug fixing
- When given a bug report: Just fix it.  Don't ask for hand-holding
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

### 7. Host storage compatibility release notes
- Every release entry in `CHANGELOG.md` must contain a
  `Host storage compatibility` subsection. Write `None` when the release has no
  relevant change; do not omit the subsection.
- For every session or vector persistence change, record the affected seam, the
  old and new logical shape or lexical behavior, whether a host migration is
  required, and whether the change is bundled-SQLite-only or host-visible.
- Treat tables, columns, serialized codec vocabulary, canonical lexical forms,
  and derived persistent identifiers as compatibility-sensitive shape.
- Copy the subsection into the annotated release-tag notes.

## Commands

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the first three; all must pass before a change is done.

## Tests

`tests/cases.rs` is the acceptance table: words that plain suffix rules get wrong. Add a row
for every bug fixed. A word with two correct answers ("leaves": leaf or leave) stays out of
the table until the crate has a stated policy for it.

## Development practices

- **Plan first** for anything non-trivial (3+ steps or a design decision); re-plan if
  something goes wrong rather than pressing on.
- **Verify before done**: tests run and pass, behaviour demonstrated.
- **Simplicity first**: minimal change, root causes not workarounds.
- **Commit only when asked.** No `Co-Authored-By` trailers.

<!-- moosedev:begin — MOOSEDev project-memory workflow. Managed by `moosedev init`; edit around this block freely, or delete the whole begin…end block to opt out. -->
> This project uses the **MOOSEDev** MCP server for durable, structured, long-term memory.
> The typed **project knowledge graph is the source of truth** for architectural decisions,
> lessons, constraints, requirements, and patterns — **not** markdown files. Free-text notes
> (e.g. `tasks/lessons.md`, `tasks/todo.md`) are optional human-readable mirrors, never canonical.

## Working with project memory (MOOSEDev)

When the `moosedev` MCP tools are available, prefer them over re-deriving context from scratch.
The loop:

1. **Recall first.** Before non-trivial work, surface prior decisions/lessons/constraints from
   the graph — and show the queries you ran. "Recall first" means a **list-all**
   `get_relevant_context` (no `topic`), not only a topic probe: a topic-scoped empty result
   means nothing cleared the relevance floor, **not** that the graph is empty.
2. **Capture as typed records.** Record durable knowledge as you go with
   `record_important_decision` (pick the right `kind`). Capture the decision **and its
   rationale** (and the rejected alternative), not transient chatter. Always report what was
   written (kind / title / returned IRI) — no silent writes.
3. **Align before coining.** Run `align_concepts` (or `suggest_mappings`) before introducing a
   new term, so the graph doesn't drift.
4. **Correct, don't duplicate.** `supersede_decision` when there is a replacement;
   `retract_decision` to deprecate one without a successor. Never silently duplicate — recall
   (list-all) first to confirm a record is genuinely new.
5. **Validate.** Run `validate_against_architecture` after capturing; resolve violations.

### Tool-selection ladder (cheap → precise)
- `get_relevant_context` — fast, deterministic, **shallow** lexical anchor/browse. Start here.
- `query` — walk-planned, synthesized natural-language answer **with a reasoning trace**. Use
  when you need reasoning over relationships, not just a label match. Keep questions short and
  focused (one question per call).
- `sparql` — exact, deterministic structural reads of the graph. Use for precise listings.

### Capture kinds
`ArchitecturalDecision` (the default — a choice + why + what was rejected) ·
`Constraint` (a hard limit/invariant) · `Requirement` (a goal/need) ·
`Pattern` (a deliberate recurring approach) · `AntiPattern` (something to avoid, + why) ·
`Lesson` (a non-obvious learning/gotcha).
<!-- moosedev:end -->
