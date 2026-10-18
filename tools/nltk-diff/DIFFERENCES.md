# NLTK differential

Dev-only comparison of moult's `singularize` against NLTK's
`WordNetLemmatizer().lemmatize(w, "n")` on Princeton WordNet 3.0. It exists to
find moult bugs, not to match NLTK: NLTK's lemmatizer applies detachment rules and
returns the shortest candidate that is in WordNet, with no policy.

## How to run

From the repository root (the first run downloads WordNet 3.0 into the
git-ignored `tools/nltk-diff/.cache/`):

    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py            # rewrite expected.tsv, expected-guess.tsv and this file
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --guess    # rewrite only expected-guess.tsv
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --check    # fail if either expected file would change

moult is called through `cargo xtask-inflect` (words on stdin, TSV on stdout),
and with `--guess` for the opt-in mode. `expected.tsv` is the default mode, the
guarantee: `singularize` never guesses. `expected-guess.tsv` is
`Inflector::builder().guess_unknown(true)`, so the opt-in residual is
reproducible and reviewable.
Only lower-case words are compared. Each disagreement lands in exactly one
bucket; the rules apply in the order the buckets are listed below
(`bucket()` in `diff.py`). `expected.tsv` holds every disagreement as
`word, moult, nltk, bucket`; a change in moult behaviour shows up as a diff
there. The buckets below are described once, under the default mode; the opt-in
section lists the same buckets plus the `fallback-*` ones.

## Word list

| source | words |
|---|---|
| src/nouns.txt | 44543 |
| moult plural of each noun (unique) | 43966 |
| FORM_TO_LEMMA forms | 1355 |
| /usr/share/dict/words, lower-cased (sha256 `be41ad97963bf8dabedd5871d5d691596175269d540956b0f9965a885c2bbab9`) | 234456 |
| union | 287329 |
| union, lower-case words compared | 287329 |

The committed expected.tsv and expected-guess.tsv were produced on macOS's `/usr/share/dict/words` (sha256 `be41ad97963bf8dabedd5871d5d691596175269d540956b0f9965a885c2bbab9`). `--check` refuses, with a message, when that file is absent or its hash differs, rather than comparing a different word set.

## Result: default mode

The default mode is the guarantee: `singularize` never guesses, so a word that no dictionary-checked step resolves is unchanged. The `fallback-*` buckets can then hold only rows where NLTK moves a word moult leaves alone.

NLTK 3.9.2, WordNet 3.0. Agreement: 285319 / 287329 = 99.30%. Disagreements: 2010.

| bucket | count |
|---|---|
| invariant | 38 |
| policy-choice | 103 |
| irregular-form | 33 |
| names | 22 |
| extra-rule | 582 |
| min-stem | 91 |
| data-version | 1026 |
| nltk-shortest-candidate | 8 |
| nltk-exception-only | 22 |
| fallback-other | 61 |
| nltk-morphy-quirk | 22 |
| different-lemma | 2 |

#### invariant (38)

The word is on moult's closed INVARIANT list (zero plurals, mass nouns, fields of study in -ics). Policy is to leave it alone; NLTK strips the final s whenever the result is in WordNet.

| word | moult | nltk |
|---|---|---|
| bellows | bellows | bellow |
| jeans | jeans | jean |
| pliers | pliers | plier |
| pragmatics | pragmatics | pragmatic |
| propaedeutics | propaedeutics | propaedeutic |
| shambles | shambles | shamble |
| shorts | shorts | short |
| species | species | specie |
| statics | statics | static |
| thrips | thrips | thrip |

#### policy-choice (103)

A word with more than one reading where the crate states a policy: SINGULAR_CHOICE ("leaves" -> "leaf", "cookies" -> "cookie"), MAN_REGULAR nouns ("human" is not a -man plural), and the checked -men/-man and -ses/-sis rules NLTK does not have. NLTK has no such choice and takes the shortest candidate.

| word | moult | nltk |
|---|---|---|
| bogies | bogie | bogy |
| caddies | caddie | caddy |
| corrigenda | corrigendum | corrigenda |
| entozoa | entozoon | entozoan |
| epizoa | epizoon | epizoan |
| halteres | haltere | halter |
| mycobacteria | mycobacterium | mycobacteria |
| oesophagi | oesophagus | esophagus |
| ploughmen | ploughman | plowman |
| stymies | stymie | stymy |

#### irregular-form (33)

A Latin/Greek or other irregular plural that OEWN lists as a form of a lemma (or the IRREGULAR supplement: person/people, bacterium/bacteria); NLTK's exception list lacks the pair and returns the word unchanged.

| word | moult | nltk |
|---|---|---|
| anthozoa | anthozoan | anthozoa |
| bacteria | bacterium | bacteria |
| hymenia | hymenium | hymenia |
| hymenoptera | hymenopteran | hymenoptera |
| lepidoptera | lepidopteran | lepidoptera |
| lumbi | lumbus | lumbi |
| magi | magus | magi |
| parazoa | parazoan | parazoa |
| people | person | people |
| tyros | tyro | tiro |

#### names (22)

Words in moult's names list that are not common nouns. moult treats them as already singular names; NLTK has no name list and may strip a final s.

| word | moult | nltk |
|---|---|---|
| as | as | a |
| briss | briss | bris |
| cheremiss | cheremiss | cheremis |
| es | es | e |
| hasidim | hasidim | hasid |
| hollands | hollands | holland |
| mls | mls | ml |
| mrs | mrs | mr |
| pekingese | pekingese | pekinese |
| scots | scots | scot |

#### extra-rule (582)

The checked -men -> -man and -ses -> -sis rules ("businessmen", "fibroses"); NLTK has a -men rule only through its exception list and no -ses -> -sis rule.

| word | moult | nltk |
|---|---|---|
| abiogeneses | abiogenesis | abiogeneses |
| adiposes | adiposis | adiposes |
| apophases | apophasis | apophases |
| bilharziases | bilharziasis | bilharziases |
| headwomen | headwoman | headwomen |
| linewomen | linewoman | linewomen |
| lobsterwomen | lobsterwoman | lobsterwomen |
| osmoses | osmosis | osmoses |
| sideswomen | sideswoman | sideswomen |
| uranalyses | uranalysis | uranalyses |

#### min-stem (91)

moult refuses a rule output shorter than three characters ("as" is not the plural of "a"); NLTK has no such floor.

| word | moult | nltk |
|---|---|---|
| bbs | bbs | bb |
| bms | bms | bm |
| dgs | dgs | dg |
| ens | ens | en |
| exes | exes | ex |
| ges | ges | ge |
| mgs | mgs | mg |
| pas | pas | pa |
| pus | pus | pu |
| yrs | yrs | yr |

#### data-version (1026)

(Compared against common lower-case lemmas only; capitalised PWN lemmas are tracked separately.) One side's answer exists only in one dictionary: Open English WordNet 2025 (moult) versus Princeton WordNet 3.0 (NLTK). Neither is wrong; the dictionaries differ.

| word | moult | nltk |
|---|---|---|
| coquets | coquet | coquets |
| devilishnesses | devilishness | devilishnesses |
| ell-wands | ell-wand | ell-wands |
| hornworks | hornwork | hornworks |
| monometallisms | monometallism | monometallisms |
| more-movers | more-mover | more-movers |
| neobaroques | neobaroque | neobaroques |
| orographs | orograph | orographs |
| pasteurisers | pasteuriser | pasteurisers |
| woodrats | woodrat | woodrats |

#### nltk-shortest-candidate (8)

The word is a noun and moult keeps it, but its stripped form is also a noun and NLTK's lemmatize() returns the shortest candidate. moult only strips through checked rules after the curated lists; keeping a known word is the conservative result.

| word | moult | nltk |
|---|---|---|
| buss | buss | bus |
| camass | camass | camas |
| canvass | canvass | canvas |
| coss | coss | cos |
| crying | crying | cry |
| cutlass | cutlass | cutlas |
| pass | pass | pas |
| puss | puss | pus |

#### nltk-exception-only (22)

NLTK reaches a PWN noun through its exception list (or a detachment chain) that moult's dictionary and rules do not; moult leaves the unknown word unchanged, the conservative result.

| word | moult | nltk |
|---|---|---|
| andtheridia | andtheridia | antheridium |
| araglis | araglis | argali |
| diaerses | diaerses | diaeresis |
| diathses | diathses | diathesis |
| filariiae | filariiae | filaria |
| gembucks | gembucks | gemsbuck |
| kumiss | kumiss | kumis |
| pess | pess | pes |
| siss | siss | sis |
| therses | therses | thyrse |

#### fallback-other (61)

Remaining disagreements on words moult's dictionary does not settle: moult keeps the word (or, in the opt-in mode, lightly strips it), NLTK answers with a PWN name or an exception-list lemma.

| word | moult | nltk |
|---|---|---|
| ashkenazim | ashkenazim | ashkenazi |
| baris | baris | bari |
| bolsheviki | bolsheviki | bolshevik |
| broadways | broadways | broadway |
| mithras | mithras | mithra |
| ranunculi | ranunculi | ranunculus |
| reguli | reguli | regulus |
| tates | tates | tate |
| thraces | thraces | thrace |
| utas | utas | uta |

#### nltk-morphy-quirk (22)

NLTK's output is not a WordNet noun or is an artefact of its detachment rules and exception lists (it returns the word's own entry when no candidate hits). moult only outputs dictionary-checked nouns.

| word | moult | nltk |
|---|---|---|
| fess | fess | fes |
| graves | grave | graf |
| grazes | graze | graz |
| ingress | ingress | ingres |
| judas | judas | juda |
| mons | mons | mon |
| mores | mores | more |
| plexiglass | plexiglass | plexiglas |
| raves | rave | raf |
| teras | teras | tera |

#### different-lemma (2)

Both outputs are WordNet nouns but different ones; the word has more than one plausible reduction and moult's order of rules differs from morphy's shortest-candidate choice.

| word | moult | nltk |
|---|---|---|
| cryings | crying | cry |
| touracos | touraco | turaco |

#### bug (0)

Unexplained residue: every row, none judged intentional. The ordered rules are broad, so an empty bucket does not mean every row is right.

| word | moult | nltk |
|---|---|---|

## Result: opt-in `guess_unknown(true)`

The same comparison with the unknown-word fallback on (`cargo xtask-inflect --guess`; rows in `expected-guess.tsv`). Every row that differs from the default mode is a word the fallback changed. These are best-effort guesses for words WordNet lacks, and the suspected defects below are the known residual of the opt-in.

NLTK 3.9.2, WordNet 3.0. Agreement: 284961 / 287329 = 99.18%. Disagreements: 2368.

| bucket | count |
|---|---|
| invariant | 38 |
| policy-choice | 103 |
| irregular-form | 33 |
| names | 22 |
| extra-rule | 582 |
| min-stem | 89 |
| data-version | 1026 |
| nltk-shortest-candidate | 8 |
| fallback-ies | 2 |
| fallback-es | 4 |
| fallback-s-after-e | 208 |
| fallback-s | 150 |
| nltk-exception-only | 21 |
| fallback-other | 58 |
| nltk-morphy-quirk | 22 |
| different-lemma | 2 |

#### invariant (38)

| word | moult | nltk |
|---|---|---|
| bellows | bellows | bellow |
| jeans | jeans | jean |
| pliers | pliers | plier |
| pragmatics | pragmatics | pragmatic |
| propaedeutics | propaedeutics | propaedeutic |
| shambles | shambles | shamble |
| shorts | shorts | short |
| species | species | specie |
| statics | statics | static |
| thrips | thrips | thrip |

#### policy-choice (103)

| word | moult | nltk |
|---|---|---|
| bogies | bogie | bogy |
| caddies | caddie | caddy |
| corrigenda | corrigendum | corrigenda |
| entozoa | entozoon | entozoan |
| epizoa | epizoon | epizoan |
| halteres | haltere | halter |
| mycobacteria | mycobacterium | mycobacteria |
| oesophagi | oesophagus | esophagus |
| ploughmen | ploughman | plowman |
| stymies | stymie | stymy |

#### irregular-form (33)

| word | moult | nltk |
|---|---|---|
| anthozoa | anthozoan | anthozoa |
| bacteria | bacterium | bacteria |
| hymenia | hymenium | hymenia |
| hymenoptera | hymenopteran | hymenoptera |
| lepidoptera | lepidopteran | lepidoptera |
| lumbi | lumbus | lumbi |
| magi | magus | magi |
| parazoa | parazoan | parazoa |
| people | person | people |
| tyros | tyro | tiro |

#### names (22)

| word | moult | nltk |
|---|---|---|
| as | as | a |
| briss | briss | bris |
| cheremiss | cheremiss | cheremis |
| es | es | e |
| hasidim | hasidim | hasid |
| hollands | hollands | holland |
| mls | mls | ml |
| mrs | mrs | mr |
| pekingese | pekingese | pekinese |
| scots | scots | scot |

#### extra-rule (582)

| word | moult | nltk |
|---|---|---|
| abiogeneses | abiogenesis | abiogeneses |
| adiposes | adiposis | adiposes |
| apophases | apophasis | apophases |
| bilharziases | bilharziasis | bilharziases |
| headwomen | headwoman | headwomen |
| linewomen | linewoman | linewomen |
| lobsterwomen | lobsterwoman | lobsterwomen |
| osmoses | osmosis | osmoses |
| sideswomen | sideswoman | sideswomen |
| uranalyses | uranalysis | uranalyses |

#### min-stem (89)

| word | moult | nltk |
|---|---|---|
| bbs | bbs | bb |
| bis | bis | bi |
| bms | bms | bm |
| dgs | dgs | dg |
| ens | ens | en |
| fts | fts | ft |
| gms | gms | gm |
| mms | mms | mm |
| pis | pis | pi |
| rus | rus | ru |

#### data-version (1026)

| word | moult | nltk |
|---|---|---|
| devilishnesses | devilishness | devilishnesses |
| ell-wands | ell-wand | ell-wands |
| hornworks | hornwork | hornworks |
| monometallisms | monometallism | monometallisms |
| more-movers | more-mover | more-movers |
| neobaroques | neobaroque | neobaroques |
| orographs | orograph | orographs |
| pasteurisers | pasteuriser | pasteurisers |
| teratons | teraton | teratons |
| woodrats | woodrat | woodrats |

#### nltk-shortest-candidate (8)

| word | moult | nltk |
|---|---|---|
| buss | buss | bus |
| camass | camass | camas |
| canvass | canvass | canvas |
| coss | coss | cos |
| crying | crying | cry |
| cutlass | cutlass | cutlas |
| pass | pass | pas |
| puss | puss | pus |

#### fallback-ies (2)

| word | moult | nltk |
|---|---|---|
| outboundaries | outboundary | outboundaries |
| sherryvallies | sherryvally | sherryvallies |

#### fallback-es (4)

| word | moult | nltk |
|---|---|---|
| autocomplexes | autocomplex | autocomplexes |
| carritches | carritch | carritches |
| gamashes | gamash | gamashes |
| gramashes | gramash | gramashes |

#### fallback-s-after-e (208)

| word | moult | nltk |
|---|---|---|
| aftertimes | aftertime | aftertimes |
| ascones | ascone | ascones |
| bettergates | bettergate | bettergates |
| brevirostrines | brevirostrine | brevirostrines |
| impages | impage | impages |
| otherwhiles | otherwhile | otherwhiles |
| scansores | scansore | scansores |
| scarpines | scarpine | scarpines |
| sequestratrices | sequestratrice | sequestratrices |
| sycones | sycone | sycones |

#### fallback-s (150)

| word | moult | nltk |
|---|---|---|
| grovelings | groveling | grovelings |
| hinderlands | hinderland | hinderlands |
| lipperings | lippering | lipperings |
| pachons | pachon | pachons |
| sexagesimals | sexagesimal | sexagesimals |
| skewings | skewing | skewings |
| tapeats | tapeat | tapeats |
| tiddledywinks | tiddledywink | tiddledywinks |
| tileworks | tilework | tileworks |
| yellowshanks | yellowshank | yellowshanks |

#### nltk-exception-only (21)

| word | moult | nltk |
|---|---|---|
| alas | alas | ala |
| araglis | araglis | argali |
| bicennaries | bicennaries | bicentenary |
| cercariiae | cercariiae | cercaria |
| diathses | diathses | diathesis |
| glochidcia | glochidcia | glochidium |
| kumiss | kumiss | kumis |
| rhachises | rhachises | rachis |
| stotkini | stotkini | stotinka |
| succuss | succuss | succus |

#### fallback-other (58)

| word | moult | nltk |
|---|---|---|
| ates | ates | ate |
| blackfeet | blackfeet | blackfoot |
| bolsheviki | bolsheviki | bolshevik |
| cebus | cebus | cebu |
| diss | diss | dis |
| gembucks | gembuck | gemsbuck |
| germen | germen | german |
| ibo | ibo | igbo |
| semitics | semitics | semitic |
| soss | soss | sos |

#### nltk-morphy-quirk (22)

| word | moult | nltk |
|---|---|---|
| arses | arse | arses |
| boss | boss | bos |
| doses | dose | dos |
| eases | ease | eas |
| fess | fess | fes |
| grazes | graze | graz |
| levis | levis | levi |
| raves | rave | raf |
| ruses | ruse | rus |
| sass | sass | sas |

#### different-lemma (2)

| word | moult | nltk |
|---|---|---|
| cryings | crying | cry |
| touracos | touraco | turaco |

#### bug (0)

Unexplained residue: every row, none judged intentional. The ordered rules are broad, so an empty bucket does not mean every row is right.

| word | moult | nltk |
|---|---|---|

## Suspected defects in the opt-in fallback (known residual)

These apply only with `guess_unknown(true)`; the default mode never guesses, so
none of them can occur there (the default-mode `fallback-*` buckets for the
unchecked rules are empty). Rows where the opt-in fallback produces a string that
is not an OEWN noun are bucketed as `fallback-*`, because the opt-in is
documented policy.

The tightened fallback (it now refuses `-as -is -us -ss -os -ys -ics -ities -wards
-selves -ops`, a vowel + `-ses`, `-sses`, `-etes`, the taxon endings `-ales -iformes
-ides`, "facies", and compounds of an invariant noun) removed 1280 rows from the
previous run (49 + 28 + 1512 + 927 before; 30 + 14 + 727 + 465 now in the four
fallback buckets). Fixed, and no longer listed:

- `-ics`, `-ops`, `-wards` words ("backwards", "afterwards", "echinops").
- `-os` and `-ys` Greek singulars and genus names ("tholos", "xyrichthys").
- `-ales`, `-iformes`, `-ides` orders and taxa ("agaricales").
- `-ities` mass nouns ("calvities", "nigrities"), "facies" and compounds of
  "species" and "series" ("superspecies", "subseries").
- Names in `-ses` and `-sses` ("moses", "ulysses", "rameses"), and `-etes` ("kubernetes").

Cost of the tightening: about 1280 unknown words that NLTK also leaves alone are
now left alone by moult too. Reading the removed rows, nearly all were non-plurals
(taxa, Greek and Latin singulars, adverbs); the genuine regular plurals now
unchanged are a handful of compounds such as "blooddrops", "beechdrops" and
"pinedrops". The 8 new rows (a word NLTK maps to a name or a PWN noun, moult now
keeps) are "broadways", "demotics", "galways", "germanics", "saccos", "semitics",
"thos" and "oses".

The known-head check (a fallback reduction is accepted only when the shortened
word ends in an OEWN noun of at least three letters, the whole word or a proper
suffix) then removed a further 634 rows from the four fallback buckets (30 + 14 +
727 + 465 before; 4 + 5 + 318 + 275 now) and fixed the non-words that were listed
here: "abies" -> "aby", "colluvies", "balbuties", "hirsuties", "antirabies",
"consocies", "macies" (the Latin `-ies` singulars); "lichenes", "gekkones",
"ascolichenes", "clavicornes", "dicotyledones" (Latin `-es` plurals);
"laches" -> "lach", "natchitoches", "albanenses". Nine rows moved into
`nltk-exception-only` and `fallback-other` (moult now leaves a word unchanged
that NLTK reduces).

The head check was then tightened again: a reduction is accepted only when the
whole result is an OEWN noun, or a proper suffix of at least four letters is a
noun with at least two letters before it (a plausible compound boundary). This
removed the short-head false positives ("elaps" -> "elap" via "lap", "perhaps"
-> "perhap" via "hap") and moved the four fallback buckets from 4 + 5 + 313 +
269 to 2 + 4 + 208 + 150 rows.

Still suspected, and not fixed (the opt-in is best-effort for words WordNet lacks):

- A compound whose last part is a noun reduces to a non-word: "afterhours" ->
  "afterhour", "belowstairs" -> "belowstair". The default mode leaves both alone
  (`tests/cases.rs`); the opt-in output is pinned as a known limitation in
  `tests/guess.rs`.
- The head check is weak where a longer noun happens to end the word: the
  reduction keeps a non-word whose last four or more letters are a noun
  ("alabastrites" -> "alabastrite" via "rite"), so `fallback-s-after-e` can
  still hold a Latin plural or a mineral name whose reduction ends in a
  four-letter noun. A sample of 45 of the 422 remaining fallback rows shows
  about a dozen non-words of that shape ("aerogene", "raptore", "traditore",
  "carnifice", "perates", "helode", "impenne", "johnadream", "chanking").
- A handful of `-ies` words ("andries" -> "andry", "manyplies" -> "manyply",
  "outboundaries", "sherryvallies") and `-ches` / `-shes` compounds
  ("carritches", "gamashes") reduce to a string that is not a noun but ends
  in one.
- `min-stem` blocks reductions to real two-letter nouns ("ads", "ids", "pis")
  because of the 3-letter floor: a deliberate cost of the floor.
