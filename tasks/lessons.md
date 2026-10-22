# Lessons

Mirror of MOOSEDev Lesson records (the graph is canonical).

## Measure a class before curating it
- **Pattern:** proposing a hand-curated list (or a generator refusal) for a whole class of
  words before counting how big that class is in the actual data.
- **Rule:** count each class in the pinned data first. Let a default rule cover the bulk;
  curate or refuse only the genuinely ambiguous rows (tens to low hundreds). If an ambiguous
  class runs into the thousands, pick a structural tie-break instead of a list.
- **Origin:** first P1 draft would have refused every word where both the input and its
  stripped form are WordNet nouns (~460 rows, growing with plural-form lemmas such as arms,
  goods, customs).

## A word list will never be complete
- **Pattern:** answering each new counterexample from an open-ended failure class (non-words
  from guessing at words outside the dictionary) by extending a refusal list or tightening a
  heuristic — four review rounds, a new counterexample each time.
- **Rule:** when the failure class is open-ended, stop patching lists. Pick a structural answer:
  a safe default that never guesses, plus an explicit opt-in for callers who prefer recall.
  Recognise the loop after the second round of the same class, and put the choice to the user.
- **Origin:** user correction, moult 0.1 review round 4 (afterhours → afterhour). The fallback
  became opt-in (`Inflector::builder().guess_unknown(true)`).

## Background CLI runs need closed stdin, and a liveness check
- **Pattern:** `codex exec "<prompt>"` launched as a background job with stdin left open printed
  "Reading additional input from stdin..." and waited for hours; nothing checked it was making
  progress, so the user had to ask.
- **Rule:** run non-interactive CLIs in the background with `< /dev/null`, and check the output
  shows real progress shortly after launch instead of only waiting for the exit notification.
  Judge progress by the output growing past the banner: codex prints "Reading additional input
  from stdin..." as its first line even when stdin is closed, so that line alone is not a hang.
- **Origin:** user correction, moult 0.2.0 review round 1 ("are they done, it's been hours").
