//! `cargo xtask report`: ambiguity class sizes and the unclassified entries.

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::classes::{Analysis, Entry};
use crate::manifest::Manifest;
use crate::wordnet::Wordnet;

const SAMPLE: usize = 30;

fn sample<T>(out: &mut String, title: &str, items: &[T], fmt: impl Fn(&T) -> String) {
    let _ = writeln!(
        out,
        "  {title} (first {} of {}):",
        SAMPLE.min(items.len()),
        items.len()
    );
    for item in items.iter().take(SAMPLE) {
        let _ = writeln!(out, "    {}", fmt(item));
    }
}

pub fn render(manifest: &Manifest, wn: &Wordnet, an: &Analysis) -> String {
    let mut out = String::new();
    let w = &mut out;
    let _ = writeln!(
        w,
        "source:  {} {} ({})",
        manifest.name, manifest.edition, manifest.license
    );
    let _ = writeln!(w, "sha256:  {}", manifest.sha256);
    let s = &wn.stats;
    let _ = writeln!(w, "\n== tables ==");
    let _ = writeln!(w, "noun entries:           {}", s.noun_entries);
    let _ = writeln!(w, "multi-word (dropped):   {}", s.multi_word);
    let _ = writeln!(w, "lemmas with '_':        {}", s.underscore);
    let _ = writeln!(w, "NOUNS:                  {}", wn.nouns.len());
    let _ = writeln!(w, "NAMES:                  {}", wn.names.len());
    let _ = writeln!(w, "noun forms (all):       {}", s.noun_forms_total);
    let _ = writeln!(
        w,
        "forms (single-token, distinct pairs): {}",
        wn.forms.len()
    );

    let unclassified = |v: &[Entry]| v.iter().filter(|e| !e.classified).count();
    let _ = writeln!(w, "\n== classes ==  (total / unclassified)");
    let _ = writeln!(
        w,
        "A  input and stripped form both nouns: {} ({} already invariant)",
        an.a.len(),
        an.a_invariant
    );
    for (label, v) in [
        ("B1 two lemmas share one regular plural", &an.b1),
        ("B2 one form maps to 2+ lemmas", &an.b2),
        ("C  irregular form that is itself a noun", &an.c),
        ("D  lemma with an ambiguous plural", &an.d),
    ] {
        let _ = writeln!(w, "{label}: {} / {}", v.len(), unclassified(v));
    }
    let total = an.b1.len() + an.b2.len() + an.c.len() + an.d.len();
    let _ = writeln!(w, "B1+B2+C+D: {total}");

    let _ = writeln!(w, "\n== other counts ==");
    let _ = writeln!(
        w,
        "NAMES-induced collisions avoided: {}",
        an.names_collisions.len()
    );
    let _ = writeln!(
        w,
        "-o lemmas with -oes form: {} of {}",
        an.o_with_oes, an.o_lemmas
    );
    let _ = writeln!(
        w,
        "-f/-fe lemmas with -ves form: {} of {}",
        an.f_with_ves, an.f_lemmas
    );
    let _ = writeln!(
        w,
        "class D(b), only form is the doubled plural: {}",
        an.d_doubling.len()
    );

    let _ = writeln!(
        w,
        "plural-shaped noun lemmas left unchanged by pluralize: {}",
        an.plural_only.len()
    );

    let _ = writeln!(w, "\n== samples ==");
    sample(w, "class A", &an.a, |(i, s)| format!("{i} -> {s}"));
    sample(
        w,
        "NAMES collisions",
        &an.names_collisions,
        |(x, with, without)| {
            let show = |s: &str| {
                if s.is_empty() {
                    "(none)".to_string()
                } else {
                    s.to_string()
                }
            };
            format!("{x}: {} instead of {}", show(with), show(without))
        },
    );
    sample(w, "plural-shaped, unchanged", &an.plural_only, |x| {
        x.clone()
    });
    let _ = writeln!(w, "  class D(b) entries: {}", an.d_doubling.join(" "));

    let _ = writeln!(w, "\n== unclassified (paste into src/policy.rs) ==");
    let mut seen = BTreeSet::new();
    for (name, list, entries) in [
        ("SINGULAR_CHOICE", "B1", &an.b1),
        ("SINGULAR_CHOICE", "B2", &an.b2),
        ("SINGULAR_CHOICE", "C", &an.c),
        ("PLURAL_CHOICE", "D", &an.d),
    ] {
        let _ = writeln!(w, "\n// {name}, class {list}");
        for e in entries.iter().filter(|e| !e.classified) {
            if e.class != crate::classes::Class::D && !seen.insert(e.input.clone()) {
                let _ = writeln!(w, "// {} {:?}: see earlier line", e.class.label(), e.input);
                continue;
            }
            let _ = writeln!(
                w,
                "    ({:?}, {:?}), // {}: {}",
                e.input,
                e.options[0],
                e.class.label(),
                e.comment
            );
        }
    }

    let _ = writeln!(w, "\n== stale or invalid policy entries ==");
    if an.problems.is_empty() {
        let _ = writeln!(w, "none");
    }
    for p in &an.problems {
        let _ = writeln!(w, "{p}");
    }
    out
}
