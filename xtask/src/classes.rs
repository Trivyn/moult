//! Ambiguity analysis: where the rules and WordNet's forms leave more than
//! one answer, and which of those the policy lists already settle.

use std::collections::{BTreeMap, BTreeSet};

use crate::policy;
use crate::rules::{checked_singular, has_plural_shape, plural};
use crate::wordnet::Wordnet;

/// The policy lists the analysis consults.
pub struct Policy<'a> {
    pub invariant: &'a [&'a str],
    pub irregular: &'a [(&'a str, &'a str)],
    pub singular_choice: &'a [(&'a str, &'a str)],
    pub plural_choice: &'a [(&'a str, &'a str)],
    pub excluded: &'a [(&'a str, &'a str)],
    pub man_regular: &'a [&'a str],
    pub ch_regular: &'a [&'a str],
    pub s_regular: &'a [&'a str],
}

impl Policy<'static> {
    pub fn builtin() -> Policy<'static> {
        Policy {
            invariant: policy::INVARIANT,
            irregular: policy::IRREGULAR,
            singular_choice: policy::SINGULAR_CHOICE,
            plural_choice: policy::PLURAL_CHOICE,
            excluded: policy::EXCLUDED_FORMS,
            man_regular: policy::MAN_REGULAR,
            ch_regular: policy::CH_REGULAR,
            s_regular: policy::S_REGULAR,
        }
    }
}

impl Policy<'_> {
    fn has_invariant(&self, w: &str) -> bool {
        self.invariant.contains(&w)
    }

    /// The plural the rules give `lemma` (the runtime's fallback in pluralize).
    fn rule_plural(&self, wn: &Wordnet, lemma: &str) -> String {
        plural(
            lemma,
            |w| wn.is_noun(w),
            |w| self.man_regular.contains(&w),
            |w| self.ch_regular.contains(&w),
        )
    }

    fn singular_choice(&self, w: &str) -> bool {
        self.singular_choice.iter().any(|(k, _)| *k == w)
    }

    /// Whether any list settles singularizing `input`.
    fn settles_singular(&self, input: &str) -> bool {
        self.has_invariant(input)
            || self.singular_choice(input)
            || self.irregular.iter().any(|(_, p)| *p == input)
    }

    /// Whether singularize keeps the known noun `w`, by the runtime's order:
    /// the invariant list, the irregular plurals, the singular choices, the
    /// WordNet forms, and the checked rules; a noun none of them reduces is
    /// kept.
    fn keeps_noun(&self, wn: &Wordnet, w: &str) -> bool {
        if self.has_invariant(w) {
            return true;
        }
        if self.irregular.iter().any(|(_, p)| *p == w) {
            return false;
        }
        if let Some((_, out)) = self.singular_choice.iter().find(|(k, _)| *k == w) {
            return *out == w;
        }
        if let Some((_, lemma)) = wn
            .forms
            .iter()
            .find(|(f, l)| f == w && !self.excluded.contains(&(l.as_str(), f.as_str())))
        {
            return lemma == w;
        }
        checked_singular(w, |x| wn.is_noun(x), |x| self.man_regular.contains(&x)).is_none()
    }

    /// Whether WordNet lists a plural form for `lemma` that is not excluded.
    fn has_listed_form(&self, wn: &Wordnet, lemma: &str) -> bool {
        wn.forms
            .iter()
            .any(|(f, l)| l == lemma && !self.excluded.contains(&(l.as_str(), f.as_str())))
    }

    fn settles_plural(&self, lemma: &str) -> bool {
        self.has_invariant(lemma)
            || self.plural_choice.iter().any(|(k, _)| *k == lemma)
            || self.irregular.iter().any(|(s, _)| *s == lemma)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Class {
    B1,
    B2,
    C,
    D,
}

impl Class {
    pub fn label(self) -> &'static str {
        match self {
            Class::B1 => "B1",
            Class::B2 => "B2",
            Class::C => "C",
            Class::D => "D",
        }
    }
}

/// One ambiguity: an input and the competing answers, best suggestion first.
pub struct Entry {
    pub class: Class,
    pub input: String,
    /// Candidates ordered by preference; `options[0]` is the suggestion.
    pub options: Vec<String>,
    /// The comment text after the class label.
    pub comment: String,
    pub classified: bool,
}

pub struct Analysis {
    /// `(input, singular)` for class A.
    pub a: Vec<(String, String)>,
    pub a_invariant: usize,
    pub b1: Vec<Entry>,
    pub b2: Vec<Entry>,
    pub c: Vec<Entry>,
    pub d: Vec<Entry>,
    /// `(word, answer with names, answer without)` where counting NAMES as
    /// targets would change the result.
    pub names_collisions: Vec<(String, String, String)>,
    pub o_lemmas: usize,
    pub o_with_oes: usize,
    pub f_lemmas: usize,
    pub f_with_ves: usize,
    /// Class D lemmas whose only form is the doubled-consonant plural.
    pub d_doubling: Vec<String>,
    /// Plural-shaped noun lemmas that singularize keeps and pluralize leaves
    /// unchanged ("thanks", "clothes").
    pub plural_only: Vec<String>,
    pub problems: Vec<String>,
}

fn plural_senses(n: u32) -> String {
    format!("{n} {}", if n == 1 { "sense" } else { "senses" })
}

/// Order candidates for singularizing `input`: most senses first, then the
/// smallest change in length (dropping only "s" wins), then byte order.
fn rank_by_senses(wn: &Wordnet, input: &str, mut cands: Vec<String>) -> Vec<String> {
    cands.sort_by(|a, b| {
        wn.senses_of(b)
            .cmp(&wn.senses_of(a))
            .then(
                input
                    .len()
                    .abs_diff(a.len())
                    .cmp(&input.len().abs_diff(b.len())),
            )
            .then(a.cmp(b))
    });
    cands
}

fn common_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

fn is_sorted_unique<T: Ord>(keys: impl Iterator<Item = T>) -> bool {
    let keys: Vec<T> = keys.collect();
    keys.windows(2).all(|w| w[0] < w[1])
}

pub fn analyze(wn: &Wordnet, pol: &Policy) -> Analysis {
    let is_noun = |w: &str| wn.is_noun(w);
    let man_regular = |w: &str| pol.man_regular.contains(&w);

    // Forms after the policy exclusions.
    let mut by_form: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut by_lemma: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (form, lemma) in &wn.forms {
        if pol.excluded.contains(&(lemma.as_str(), form.as_str())) {
            continue;
        }
        by_form.entry(form).or_default().insert(lemma);
        by_lemma.entry(lemma).or_default().insert(form);
    }

    // Class A: the input and its stripped form are both nouns.
    let mut a = Vec::new();
    let mut a_invariant = 0;
    for lemma in &wn.nouns {
        if let Some(s) = checked_singular(lemma, is_noun, man_regular) {
            if pol.has_invariant(lemma) {
                a_invariant += 1;
            }
            a.push((lemma.clone(), s));
        }
    }

    // Class B1: two nouns reachable from one rule plural by the rules.
    let mut plurals: BTreeSet<String> = BTreeSet::new();
    for lemma in &wn.nouns {
        plurals.insert(pol.rule_plural(wn, lemma));
    }
    let mut b1 = Vec::new();
    for p in plurals {
        let nouns: BTreeSet<String> = crate::rules::singular_candidates(&p)
            .into_iter()
            .filter(|c| is_noun(c))
            .collect();
        if nouns.len() >= 2 {
            let options = rank_by_senses(wn, &p, nouns.into_iter().collect());
            let comment = options
                .iter()
                .map(|o| format!("{o} ({})", plural_senses(wn.senses_of(o))))
                .collect::<Vec<_>>()
                .join(" | ");
            b1.push(Entry {
                class: Class::B1,
                classified: pol.settles_singular(&p),
                input: p,
                options,
                comment,
            });
        }
    }

    // Classes B2 and C, from the form list.
    let mut b2 = Vec::new();
    let mut c = Vec::new();
    for (form, lemmas) in &by_form {
        let lemmas: Vec<String> = lemmas
            .iter()
            .filter(|l| *l != form)
            .map(|l| l.to_string())
            .collect();
        if lemmas.is_empty() {
            continue;
        }
        let classified = pol.settles_singular(form);
        let options = rank_by_senses(wn, form, lemmas);
        if options.len() >= 2 {
            let comment = options
                .iter()
                .map(|o| format!("{o} ({})", plural_senses(wn.senses_of(o))))
                .collect::<Vec<_>>()
                .join(" | ");
            b2.push(Entry {
                class: Class::B2,
                input: form.to_string(),
                options: options.clone(),
                comment,
                classified,
            });
        }
        if wn.is_noun(form) {
            let mut comment = options
                .iter()
                .map(|o| format!("{o} ({})", plural_senses(wn.senses_of(o))))
                .collect::<Vec<_>>();
            comment.push(format!("{form} ({})", plural_senses(wn.senses_of(form))));
            c.push(Entry {
                class: Class::C,
                input: form.to_string(),
                options,
                comment: comment.join(" | "),
                classified,
            });
        }
    }

    // Class D: a lemma whose plural is ambiguous. (a) It has two or more
    // forms, self-forms included; or (b) its only form is the regular plural
    // with the final consonant doubled (bus, busses).
    let mut d = Vec::new();
    let mut d_doubling = Vec::new();
    for (lemma, forms) in &by_lemma {
        let doubling = |form: &str| {
            lemma
                .chars()
                .last()
                .is_some_and(|ch| form == format!("{lemma}{ch}es"))
        };
        let only_doubling = forms.len() == 1 && forms.iter().all(|f| doubling(f));
        if forms.len() < 2 && !only_doubling {
            continue;
        }
        let mut options: Vec<String> = forms.iter().map(|s| s.to_string()).collect();
        let regular = pol.rule_plural(wn, lemma);
        if !options.contains(&regular) {
            options.push(regular);
        }
        options.sort_by(|x, y| {
            common_prefix(lemma, y)
                .cmp(&common_prefix(lemma, x))
                .then(x.cmp(y))
        });
        if only_doubling {
            d_doubling.push(lemma.to_string());
        }
        d.push(Entry {
            class: Class::D,
            input: lemma.to_string(),
            comment: options.join(" | "),
            options,
            classified: pol.settles_plural(lemma),
        });
    }

    // NAMES collisions avoided.
    let mut names_collisions = Vec::new();
    for w in wn.nouns.iter().chain(wn.names.iter()) {
        let with = checked_singular(w, |x| wn.is_known(x), man_regular);
        let without = checked_singular(w, is_noun, man_regular);
        if with != without {
            names_collisions.push((
                w.clone(),
                with.unwrap_or_default(),
                without.unwrap_or_default(),
            ));
        }
    }
    names_collisions.sort();

    // -o and -f lemmas that WordNet gives an irregular plural.
    let (mut o_lemmas, mut o_with_oes, mut f_lemmas, mut f_with_ves) = (0, 0, 0, 0);
    let empty = BTreeSet::new();
    for lemma in &wn.nouns {
        let forms = by_lemma.get(lemma.as_str()).unwrap_or(&empty);
        if lemma.ends_with('o') {
            o_lemmas += 1;
            if forms.contains(format!("{lemma}es").as_str()) {
                o_with_oes += 1;
            }
        }
        if lemma.ends_with('f') || lemma.ends_with("fe") {
            f_lemmas += 1;
            if forms.iter().any(|f| f.ends_with("ves")) {
                f_with_ves += 1;
            }
        }
    }

    // Plural-shaped lemmas that nothing lists a plural for: pluralize leaves
    // them as the plural they look like.
    let plural_only: Vec<String> = wn
        .nouns
        .iter()
        .filter(|w| {
            has_plural_shape(w)
                && !pol.s_regular.contains(&w.as_str())
                && !pol.settles_plural(w)
                && !pol.has_listed_form(wn, w)
                && pol.keeps_noun(wn, w)
        })
        .cloned()
        .collect();

    let mut an = Analysis {
        a,
        a_invariant,
        b1,
        b2,
        c,
        d,
        names_collisions,
        o_lemmas,
        o_with_oes,
        f_lemmas,
        f_with_ves,
        d_doubling,
        plural_only,
        problems: Vec::new(),
    };
    an.problems = validate(wn, pol, &an);
    an
}

/// Every answer a SINGULAR_CHOICE for `input` may name: lemmas of WordNet
/// forms spelled `input` (after exclusions) and nouns the checked rules reach.
fn singular_targets(wn: &Wordnet, pol: &Policy, input: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = wn
        .forms
        .iter()
        .filter(|(f, l)| f == input && !pol.excluded.contains(&(l.as_str(), f.as_str())))
        .map(|(_, l)| l.clone())
        .collect();
    out.extend(
        crate::rules::singular_candidates(input)
            .into_iter()
            .filter(|c| wn.is_noun(c)),
    );
    out
}

/// Why `word` cannot be a policy output, or `None`: it must be non-empty and
/// made of lower-case ASCII letters, apostrophes and hyphens only (so no
/// digits, whitespace or capitals).
fn bad_shape(word: &str) -> Option<&'static str> {
    if word.is_empty() {
        Some("empty")
    } else if !word
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b == b'\'' || b == b'-')
    {
        Some("not made of lower-case letters, apostrophes and hyphens")
    } else {
        None
    }
}

/// Whether an IRREGULAR plural is a real word: either a WordNet word (noun,
/// name or other-part-of-speech lemma), or a compound of a WordNet irregular
/// form. The compound case is `singular` = P + lemma and `plural` = P + form
/// for a WordNet (form, lemma) pair, with a non-empty prefix P once any
/// trailing hyphen is dropped (schoolchild / schoolchildren via child /
/// children).
fn irregular_plural_is_word(wn: &Wordnet, pol: &Policy, singular: &str, plural: &str) -> bool {
    if wn.is_known(plural) || wn.others.contains(plural) {
        return true;
    }
    wn.forms.iter().any(|(form, lemma)| {
        !pol.excluded.contains(&(lemma.as_str(), form.as_str()))
            && singular
                .strip_suffix(lemma.as_str())
                .zip(plural.strip_suffix(form.as_str()))
                .is_some_and(|(p, q)| p == q && !p.trim_end_matches('-').is_empty())
    })
}

/// Whether dropping `(lemma, plural)` from PLURAL_CHOICE leaves the runtime
/// plural unchanged: WordNet's surviving forms for the lemma are exactly
/// `plural`. A lone doubled-consonant form (bus, busses) is kept: class D
/// requires the policy to acknowledge it.
fn plural_choice_redundant(wn: &Wordnet, pol: &Policy, lemma: &str, plural: &str) -> bool {
    let forms: BTreeSet<&str> = wn
        .forms
        .iter()
        .filter(|(f, l)| l == lemma && !pol.excluded.contains(&(l.as_str(), f.as_str())))
        .map(|(f, _)| f.as_str())
        .collect();
    let doubling = lemma
        .chars()
        .last()
        .is_some_and(|ch| plural == format!("{lemma}{ch}es"));
    forms.len() == 1 && forms.contains(plural) && !doubling
}

/// Invalid, stale, or malformed policy entries.
fn validate(wn: &Wordnet, pol: &Policy, an: &Analysis) -> Vec<String> {
    let mut out = Vec::new();
    let sorted = |name: &str, ok: bool, out: &mut Vec<String>| {
        if !ok {
            out.push(format!(
                "{name}: not sorted by first element, or duplicate key"
            ));
        }
    };
    sorted(
        "INVARIANT",
        is_sorted_unique(pol.invariant.iter()),
        &mut out,
    );
    sorted(
        "IRREGULAR",
        is_sorted_unique(pol.irregular.iter().map(|p| p.0)),
        &mut out,
    );
    sorted(
        "SINGULAR_CHOICE",
        is_sorted_unique(pol.singular_choice.iter().map(|p| p.0)),
        &mut out,
    );
    sorted(
        "PLURAL_CHOICE",
        is_sorted_unique(pol.plural_choice.iter().map(|p| p.0)),
        &mut out,
    );
    sorted(
        "EXCLUDED_FORMS",
        is_sorted_unique(pol.excluded.iter()),
        &mut out,
    );

    sorted(
        "MAN_REGULAR",
        is_sorted_unique(pol.man_regular.iter()),
        &mut out,
    );
    sorted(
        "CH_REGULAR",
        is_sorted_unique(pol.ch_regular.iter()),
        &mut out,
    );
    sorted(
        "S_REGULAR",
        is_sorted_unique(pol.s_regular.iter()),
        &mut out,
    );
    let has_form = |lemma: &str| wn.forms.iter().any(|(_, l)| l == lemma);

    for w in pol.invariant {
        if !wn.is_noun(w) {
            out.push(format!("INVARIANT {w:?}: not a lower-case WordNet noun"));
        }
    }
    for (s, p) in pol.irregular {
        if let Some(why) = bad_shape(p) {
            out.push(format!("IRREGULAR ({s:?}, {p:?}): plural is {why}"));
        } else {
            if p == s {
                out.push(format!(
                    "IRREGULAR ({s:?}, {p:?}): plural equals the singular; use INVARIANT"
                ));
            }
            if p.as_bytes()[0] != s.as_bytes()[0] {
                out.push(format!(
                    "IRREGULAR ({s:?}, {p:?}): plural does not start with the singular's letter"
                ));
            }
            if let Some((_, other)) = wn.forms.iter().find(|(f, l)| {
                f == p && l != s && !pol.excluded.contains(&(l.as_str(), f.as_str()))
            }) {
                out.push(format!(
                    "IRREGULAR ({s:?}, {p:?}): plural is a WordNet form of a different lemma, {other:?}"
                ));
            }
            if !irregular_plural_is_word(wn, pol, s, p) {
                out.push(format!(
                    "IRREGULAR ({s:?}, {p:?}): plural is not a WordNet word or a compound of a WordNet irregular form"
                ));
            }
        }
        if !wn.is_noun(s) {
            out.push(format!(
                "IRREGULAR ({s:?}, {p:?}): singular is not a lower-case WordNet noun"
            ));
        }
        if has_form(s) {
            out.push(format!(
                "IRREGULAR ({s:?}, {p:?}): stale, WordNet lists a form for {s:?}; use PLURAL_CHOICE"
            ));
        } else if pol.rule_plural(wn, s) == *p
            && checked_singular(p, |w| wn.is_noun(w), |w| pol.man_regular.contains(&w)).as_deref()
                == Some(*s)
        {
            out.push(format!(
                "IRREGULAR ({s:?}, {p:?}): redundant, the rules already give both directions"
            ));
        }
    }
    let singular_inputs: BTreeSet<&str> = an
        .b1
        .iter()
        .chain(&an.b2)
        .chain(&an.c)
        .map(|e| e.input.as_str())
        .collect();
    // A SINGULAR_CHOICE is not checked for redundancy: classes B1, B2 and C
    // demand an acknowledgement even where resolution already gives the same
    // answer, and an entry with no live ambiguity is reported stale below.
    for (input, output) in pol.singular_choice {
        if let Some(why) = bad_shape(output) {
            out.push(format!(
                "SINGULAR_CHOICE ({input:?}, {output:?}): output is {why}"
            ));
        }
        if !singular_inputs.contains(input) {
            out.push(format!(
                "SINGULAR_CHOICE ({input:?}, {output:?}): stale, no live ambiguity"
            ));
        }
        if input != output && !singular_targets(wn, pol, input).contains(*output) {
            out.push(format!(
                "SINGULAR_CHOICE ({input:?}, {output:?}): invalid, output is neither the input \
                 nor a candidate lemma"
            ));
        }
    }
    for (lemma, plural) in pol.plural_choice {
        if let Some(why) = bad_shape(plural) {
            out.push(format!(
                "PLURAL_CHOICE ({lemma:?}, {plural:?}): plural is {why}"
            ));
        }
        if !has_form(lemma) {
            out.push(format!(
                "PLURAL_CHOICE ({lemma:?}, {plural:?}): stale, WordNet lists no form for {lemma:?}"
            ));
            continue;
        }
        if plural_choice_redundant(wn, pol, lemma, plural) {
            out.push(format!(
                "PLURAL_CHOICE ({lemma:?}, {plural:?}): redundant, resolution gives the same \
                 answer without it"
            ));
        }
        let is_form = wn.forms.contains(&(plural.to_string(), lemma.to_string()));
        if !is_form && pol.rule_plural(wn, lemma) != *plural {
            out.push(format!(
                "PLURAL_CHOICE ({lemma:?}, {plural:?}): invalid, neither a WordNet form nor the \
                 rule plural"
            ));
        }
    }
    for w in pol.man_regular {
        if !(wn.is_noun(w) && w.ends_with("man")) {
            out.push(format!(
                "MAN_REGULAR {w:?}: not a lower-case WordNet noun ending in \"man\""
            ));
        }
    }
    for w in pol.ch_regular {
        if !(wn.is_noun(w) && w.ends_with("ch")) {
            out.push(format!(
                "CH_REGULAR {w:?}: not a lower-case WordNet noun ending in \"ch\""
            ));
        } else if has_form(w) || pol.settles_plural(w) {
            out.push(format!(
                "CH_REGULAR {w:?}: stale, a WordNet form or another list settles the plural"
            ));
        }
    }
    for w in pol.s_regular {
        if !(wn.is_noun(w) && has_plural_shape(w)) {
            out.push(format!(
                "S_REGULAR {w:?}: not a lower-case WordNet noun of the plural shape"
            ));
        } else if pol.singular_choice(w) {
            out.push(format!(
                "S_REGULAR {w:?}: conflicts with SINGULAR_CHOICE, which settles the singular"
            ));
        } else if !pol.keeps_noun(wn, w) {
            out.push(format!(
                "S_REGULAR {w:?}: stale, singularize reduces it, so pluralize never reaches it"
            ));
        } else if pol.has_listed_form(wn, w) || pol.settles_plural(w) {
            out.push(format!(
                "S_REGULAR {w:?}: stale, a WordNet form or another list settles the plural"
            ));
        }
    }
    for (lemma, form) in pol.excluded {
        if !wn.forms.contains(&(form.to_string(), lemma.to_string())) {
            out.push(format!(
                "EXCLUDED_FORMS ({lemma:?}, {form:?}): stale, not a WordNet form"
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wordnet() -> Wordnet {
        let mut wn = Wordnet::default();
        for n in [
            "cookie", "cooky", "bus", "leaf", "leave", "man", "men", "cry",
        ] {
            wn.nouns.insert(n.into());
            wn.senses.insert(n.into(), 1);
        }
        wn.senses.insert("cookie".into(), 3);
        for (f, l) in [
            ("leaves", "leaf"),
            ("leaves", "leave"),
            ("men", "man"),
            ("busses", "bus"),
            ("buses", "bus"),
            ("crying", "cry"),
            ("cries", "cry"),
        ] {
            wn.forms.insert((f.into(), l.into()));
        }
        wn
    }

    fn no_policy() -> Policy<'static> {
        Policy {
            invariant: &[],
            irregular: &[],
            singular_choice: &[],
            plural_choice: &[],
            excluded: &[],
            man_regular: &[],
            ch_regular: &[],
            s_regular: &[],
        }
    }

    #[test]
    fn classifies_small_wordnet() {
        let wn = wordnet();
        let an = analyze(&wn, &no_policy());
        // cookies reaches cookie and cooky; cookie wins on senses.
        let cookies = an.b1.iter().find(|e| e.input == "cookies").unwrap();
        assert_eq!(cookies.options, ["cookie", "cooky"]);
        let leaves = an.b2.iter().find(|e| e.input == "leaves").unwrap();
        assert_eq!(leaves.options, ["leave", "leaf"]);
        assert_eq!(an.c.len(), 1);
        assert_eq!(an.c[0].input, "men");
        // bus has buses and busses; cry has crying and cries.
        assert!(an.d.iter().any(|e| e.input == "bus"));
        assert!(an.d.iter().any(|e| e.input == "cry"));
        assert!(an.d_doubling.is_empty());
    }

    #[test]
    fn policy_settles_and_flags_stale() {
        let wn = wordnet();
        let pol = Policy {
            singular_choice: &[("leaves", "leaf"), ("zzz", "zzz")],
            excluded: &[("cry", "crying")],
            ..no_policy()
        };
        let an = analyze(&wn, &pol);
        assert!(
            an.b2
                .iter()
                .find(|e| e.input == "leaves")
                .unwrap()
                .classified
        );
        assert!(!an.d.iter().any(|e| e.input == "cry"));
        assert!(an.problems.iter().any(|p| p.contains("\"zzz\"")));
    }

    fn problems(wn: &Wordnet, pol: &Policy) -> Vec<String> {
        analyze(wn, pol).problems
    }

    #[test]
    fn class_d_definition() {
        let mut wn = wordnet();
        for n in ["gas", "quiz", "forceps", "genus"] {
            wn.nouns.insert(n.into());
        }
        for (f, l) in [
            ("gasses", "gas"),
            ("gas", "gas"),
            ("quizzes", "quiz"),
            ("forceps", "forceps"),
            ("genera", "genus"),
        ] {
            wn.forms.insert((f.into(), l.into()));
        }
        let an = analyze(&wn, &no_policy());
        let d = |l: &str| an.d.iter().any(|e| e.input == l);
        // (a) two forms, a self-form counting as one.
        assert!(d("gas") && d("bus"));
        // (b) the only form is the doubled plural.
        assert!(d("quiz"));
        assert_eq!(an.d_doubling, ["quiz"]);
        // A lone self-form is not ambiguous; nor is a lone irregular form.
        assert!(!d("forceps") && !d("genus") && !d("man"));
        // Settled by PLURAL_CHOICE, IRREGULAR or INVARIANT.
        let pol = Policy {
            plural_choice: &[("bus", "buses")],
            irregular: &[("gas", "gases")],
            invariant: &["quiz"],
            ..no_policy()
        };
        let an = analyze(&wn, &pol);
        for l in ["bus", "gas", "quiz"] {
            assert!(an.d.iter().find(|e| e.input == l).unwrap().classified);
        }
    }

    #[test]
    fn validates_singular_choice_output() {
        let wn = wordnet();
        let ok = Policy {
            singular_choice: &[("cookies", "cookie"), ("leaves", "leaves")],
            ..no_policy()
        };
        assert!(problems(&wn, &ok).is_empty());
        let bad = Policy {
            singular_choice: &[("cookies", "man")],
            ..no_policy()
        };
        assert!(problems(&wn, &bad).iter().any(|p| p.contains("invalid")));
    }

    #[test]
    fn validates_plural_choice_output() {
        let wn = wordnet();
        for plural in ["buses", "busses"] {
            let pol = Policy {
                plural_choice: &[("bus", plural)],
                ..no_policy()
            };
            assert!(problems(&wn, &pol).is_empty(), "{plural}");
        }
        let bad = Policy {
            plural_choice: &[("bus", "bi")],
            ..no_policy()
        };
        assert!(problems(&wn, &bad).iter().any(|p| p.contains("invalid")));
    }

    #[test]
    fn plural_choice_needs_a_wordnet_form_and_man_regular_a_noun() {
        let wn = wordnet();
        // A preference over WordNet's form is valid, with the rule plural.
        let ok = Policy {
            plural_choice: &[("leaf", "leafs")],
            ..no_policy()
        };
        assert!(problems(&wn, &ok).is_empty());
        // A lemma without any WordNet form is stale.
        let stale = Policy {
            plural_choice: &[("cry", "cries"), ("man", "mans"), ("cookie", "cookies")],
            ..no_policy()
        };
        assert!(problems(&wn, &stale)
            .iter()
            .any(|p| p.contains("\"cookie\"") && p.contains("stale")));
        let bad = Policy {
            man_regular: &["man", "cry", "shaman"],
            ..no_policy()
        };
        let p = problems(&wn, &bad);
        assert!(p.iter().any(|x| x.contains("MAN_REGULAR \"cry\"")));
        assert!(p.iter().any(|x| x.contains("MAN_REGULAR \"shaman\"")));
        assert!(!p.iter().any(|x| x.contains("MAN_REGULAR \"man\"")));
    }

    #[test]
    fn s_regular_needs_a_plural_shaped_noun_without_a_form() {
        let mut wn = wordnet();
        for n in ["lens", "thanks", "bus", "mice"] {
            wn.nouns.insert(n.into());
        }
        wn.forms.insert(("lenses".into(), "lens".into()));
        let pol = Policy {
            s_regular: &["bus", "lens", "mice", "thanks"],
            ..no_policy()
        };
        let p = problems(&wn, &pol);
        // A vowel before the s is not the plural shape; "mice" is no s-word.
        assert!(p.iter().any(|x| x.contains("S_REGULAR \"bus\": not")));
        assert!(p.iter().any(|x| x.contains("S_REGULAR \"mice\": not")));
        // A WordNet form already settles "lens"; "thanks" has none.
        assert!(p.iter().any(|x| x.contains("S_REGULAR \"lens\": stale")));
        assert!(!p.iter().any(|x| x.contains("S_REGULAR \"thanks\"")));
        // A SINGULAR_CHOICE may not also claim the word.
        let mut wn2 = wordnet();
        wn2.nouns.insert("auspices".into());
        let clash = Policy {
            s_regular: &["auspices"],
            singular_choice: &[("auspices", "auspices")],
            ..no_policy()
        };
        let p = problems(&wn2, &clash);
        assert!(p
            .iter()
            .any(|x| x.contains("S_REGULAR \"auspices\": conflicts")));
        // An entry singularize reduces ("glasses" to "glass") is stale.
        let mut wn3 = wordnet();
        for n in ["glass", "glasses"] {
            wn3.nouns.insert(n.into());
        }
        let reduced = Policy {
            s_regular: &["glasses"],
            ..no_policy()
        };
        let p = problems(&wn3, &reduced);
        assert!(p
            .iter()
            .any(|x| x.contains("S_REGULAR \"glasses\": stale, singularize")));
        // Without the entry, "thanks" is a plural-only noun.
        let an = analyze(&wn, &no_policy());
        assert_eq!(an.plural_only, ["thanks"]);
    }

    #[test]
    fn redundant_choices_are_rejected() {
        let mut wn = wordnet();
        wn.nouns.insert("mouse".into());
        wn.forms.insert(("mice".into(), "mouse".into()));
        let red = |pol: Policy, needle: &str| {
            let p = problems(&wn, &pol);
            assert!(
                p.iter()
                    .any(|x| x.contains("redundant") && x.contains(needle)),
                "{needle}: {p:?}"
            );
        };
        red(
            Policy {
                plural_choice: &[("mouse", "mice")],
                ..no_policy()
            },
            "PLURAL_CHOICE",
        );
        // A real choice among several forms is not redundant.
        let ok = Policy {
            plural_choice: &[("bus", "buses")],
            ..no_policy()
        };
        assert!(!problems(&wn, &ok).iter().any(|x| x.contains("redundant")));
        let ok = Policy {
            singular_choice: &[("leaves", "leaf")],
            ..no_policy()
        };
        assert!(!problems(&wn, &ok).iter().any(|x| x.contains("redundant")));
    }

    #[test]
    fn irregular_must_have_no_wordnet_form() {
        let wn = wordnet();
        let pol = Policy {
            irregular: &[("leaf", "leafs")],
            ..no_policy()
        };
        assert!(problems(&wn, &pol)
            .iter()
            .any(|p| p.contains("IRREGULAR") && p.contains("use PLURAL_CHOICE")));
    }

    #[test]
    fn validates_invariant_irregular_and_excluded() {
        let wn = wordnet();
        let pol = Policy {
            invariant: &["Zed", "zzz"],
            irregular: &[("man", "men"), ("zzz", "zzzs")],
            excluded: &[("cry", "crying"), ("cry", "weeping")],
            ..no_policy()
        };
        let p = problems(&wn, &pol);
        let has = |s: &str| p.iter().any(|x| x.contains(s));
        assert!(has("INVARIANT \"Zed\""));
        assert!(has("INVARIANT \"zzz\""));
        assert!(has("IRREGULAR (\"man\", \"men\"): stale"));
        assert!(has("IRREGULAR (\"zzz\", \"zzzs\"): singular"));
        assert!(has("EXCLUDED_FORMS (\"cry\", \"weeping\")"));
        assert!(!has("EXCLUDED_FORMS (\"cry\", \"crying\")"));
    }

    #[test]
    fn irregular_plural_is_validated() {
        let mut wn = wordnet();
        wn.nouns.insert("person".into());
        wn.nouns.insert("people".into());
        wn.forms.insert(("cats".into(), "cat".into()));
        let bad = |s: &'static str, p: &'static str, needle: &str| {
            let irregular = [(s, p)];
            let pol = Policy {
                irregular: &irregular,
                ..no_policy()
            };
            let found = problems(&wn, &pol);
            assert!(
                found.iter().any(|x| x.contains(needle)),
                "({s}, {p}): {found:?}"
            );
        };
        bad("person", "p3ople", "not made of");
        bad("person", "People", "not made of");
        bad("person", "peo ple", "not made of");
        bad("person", "", "empty");
        bad("person", "person", "equals the singular");
        bad("person", "human", "start with");
        bad("person", "cats", "different lemma");
        let ok = [("person", "people")];
        let pol = Policy {
            irregular: &ok,
            ..no_policy()
        };
        assert!(problems(&wn, &pol).is_empty());
    }

    #[test]
    fn irregular_plural_must_be_a_word_or_compound() {
        let mut wn = wordnet();
        wn.nouns.insert("person".into());
        wn.nouns.insert("people".into());
        wn.nouns.insert("schoolchild".into());
        wn.forms.insert(("children".into(), "child".into()));
        wn.others.insert("sunnys".into());
        let check = |s: &'static str, p: &'static str| {
            let irregular = [(s, p)];
            let pol = Policy {
                irregular: &irregular,
                ..no_policy()
            };
            problems(&wn, &pol)
                .iter()
                .any(|x| x.contains("not a WordNet word or a compound"))
        };
        // A plural that is not a word is rejected.
        assert!(check("person", "peopla"));
        // A WordNet noun is accepted.
        assert!(!check("person", "people"));
        // An other-part-of-speech lemma is accepted.
        assert!(!check("person", "sunnys"));
        // A compound of a WordNet irregular form is accepted.
        assert!(!check("schoolchild", "schoolchildren"));
        // A compound with a wrong form is rejected, and so is an empty prefix.
        assert!(check("schoolchild", "schoolchilden"));
        assert!(check("child", "children"));
    }

    #[test]
    fn choice_outputs_are_shape_checked() {
        let wn = wordnet();
        let sing = Policy {
            singular_choice: &[("leaves", "le4f")],
            ..no_policy()
        };
        assert!(problems(&wn, &sing)
            .iter()
            .any(|p| p.contains("SINGULAR_CHOICE") && p.contains("not made of")));
        let plur = Policy {
            plural_choice: &[("bus", "bu ses")],
            ..no_policy()
        };
        assert!(problems(&wn, &plur)
            .iter()
            .any(|p| p.contains("PLURAL_CHOICE") && p.contains("not made of")));
    }

    #[test]
    fn invariant_entry_is_not_stale_when_unambiguous() {
        let wn = wordnet();
        let pol = Policy {
            invariant: &["bus"],
            ..no_policy()
        };
        assert!(problems(&wn, &pol).is_empty());
    }

    #[test]
    fn requires_sorted_unique_keys() {
        let wn = wordnet();
        let pol = Policy {
            invariant: &["man", "cry"],
            irregular: &[("man", "men"), ("man", "mans")],
            singular_choice: &[("b", "b"), ("a", "a")],
            plural_choice: &[("bus", "buses"), ("bus", "busses")],
            excluded: &[("cry", "cryings"), ("cry", "crying")],
            man_regular: &["shaman", "human"],
            ch_regular: &["stomach", "monarch"],
            s_regular: &["yes", "lens"],
        };
        let p = problems(&wn, &pol);
        for list in [
            "INVARIANT",
            "IRREGULAR",
            "SINGULAR_CHOICE",
            "PLURAL_CHOICE",
            "EXCLUDED_FORMS",
            "MAN_REGULAR",
            "CH_REGULAR",
            "S_REGULAR",
        ] {
            assert!(
                p.iter()
                    .any(|x| x.starts_with(list) && x.contains("not sorted")),
                "{list}"
            );
        }
    }
}
