//! Dev-only tooling for moult. Run as `cargo xtask <command>`; never
//! published and never a dependency of the runtime crate.

extern crate alloc;

#[path = "../../src/policy.rs"]
mod policy;
#[path = "../../src/pos.rs"]
mod pos;
#[path = "../../src/rules.rs"]
mod rules;

mod classes;
mod fetch;
mod gen;
#[cfg(feature = "inflect")]
mod inflect;
mod manifest;
mod report;
mod size;
mod wordnet;

use std::path::PathBuf;
use std::process::ExitCode;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn run_report() -> Result<(), String> {
    let root = root();
    let manifest = manifest::Manifest::load(&root.join("data/MANIFEST.toml"))?;
    let archive = fetch::ensure(&manifest, &root.join("data/cache"))?;
    let wn = wordnet::parse(&archive)?;
    let analysis = classes::analyze(&wn, &classes::Policy::builtin());
    print!("{}", report::render(&manifest, &wn, &analysis));
    Ok(())
}

const USAGE: &str = "usage: cargo xtask report\n       cargo xtask gen-data [--check]\n       cargo xtask size\n       cargo xtask-inflect [--guess | --lemma | --lemma-candidates] < words.txt   (alias for `run -p xtask --features inflect -- inflect`)";

/// Generation always refuses on an unclassified, stale or invalid policy entry.
fn run_gen(check: bool) -> Result<(), String> {
    let root = root();
    let manifest = manifest::Manifest::load(&root.join("data/MANIFEST.toml"))?;
    let archive = fetch::ensure(&manifest, &root.join("data/cache"))?;
    let wn = wordnet::parse(&archive)?;
    let an = classes::analyze(&wn, &classes::Policy::builtin());
    let unclassified = [&an.b1, &an.b2, &an.c, &an.d]
        .iter()
        .map(|v| v.iter().filter(|e| !e.classified).count())
        .sum::<usize>()
        + an.l1.iter().filter(|e| !e.classified).count();
    if unclassified > 0 || !an.problems.is_empty() {
        return Err(format!(
            "refusing to generate: {unclassified} unclassified and {} stale or invalid policy entries; run `cargo xtask report` and fix src/policy.rs",
            an.problems.len()
        ));
    }
    let out = gen::render(&manifest, &wn, policy::LEMMA_EXCLUDED);
    if check {
        gen::check(&root, &out)?;
        eprintln!("generated files are up to date");
        Ok(())
    } else {
        gen::write(&root, &out)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("report") if args.len() == 1 => run_report(),
        #[cfg(feature = "inflect")]
        Some("inflect") if args.len() <= 2 => {
            match args.get(1).map(String::as_str) {
                None => inflect::run(inflect::Mode::Noun { guess: false }),
                Some("--guess") => inflect::run(inflect::Mode::Noun { guess: true }),
                Some("--lemma") => inflect::run(inflect::Mode::Lemma),
                Some("--lemma-candidates") => inflect::run(inflect::Mode::LemmaCandidates),
                Some(_) => Err(USAGE.to_string()),
            }
        }
        #[cfg(not(feature = "inflect"))]
        Some("inflect") => Err(
            "`inflect` needs the `inflect` feature (it uses the moult crate); run `cargo xtask-inflect` instead".to_string(),
        ),
        Some("size") if args.len() == 1 => size::run(),
        Some("gen-data") => {
            let flags = &args[1..];
            if flags.iter().all(|f| f == "--check") {
                run_gen(!flags.is_empty())
            } else {
                Err(USAGE.to_string())
            }
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod rules_tests {
    use crate::rules::*;

    #[test]
    fn regular_plurals() {
        for (l, p) in [
            ("city", "cities"),
            ("day", "days"),
            ("box", "boxes"),
            ("church", "churches"),
            ("dish", "dishes"),
            ("bus", "buses"),
            ("photo", "photos"),
            ("roof", "roofs"),
            ("cat", "cats"),
        ] {
            assert_eq!(regular_plural(l), p);
        }
    }

    #[test]
    fn rule_plurals() {
        let nouns = ["policeman", "human", "man", "mitosis", "sis"];
        let known = |w: &str| nouns.contains(&w);
        let man_regular = |w: &str| w == "human";
        let ch_regular = |w: &str| w == "stomach";
        let p = |l: &str| plural(l, known, man_regular, ch_regular);
        assert_eq!(p("policeman"), "policemen");
        assert_eq!(p("man"), "men");
        assert_eq!(p("human"), "humans");
        // An unknown word ending in "man" takes the regular plural.
        assert_eq!(p("spiderman"), "spidermans");
        assert_eq!(p("mitosis"), "mitoses");
        assert_eq!(p("apoptosis"), "apoptoses");
        assert_eq!(p("sis"), "sises");
        assert_eq!(p("city"), "cities");
        assert_eq!(p("stomach"), "stomachs");
        assert_eq!(p("church"), "churches");
    }

    #[test]
    fn men_never_reduces_to_a_man_regular_noun() {
        let nouns = ["dolman", "policeman"];
        let known = |w: &str| nouns.contains(&w);
        let man_regular = |w: &str| w == "dolman";
        assert_eq!(checked_singular("dolmen", known, man_regular), None);
        assert_eq!(
            checked_singular("policemen", known, man_regular).as_deref(),
            Some("policeman")
        );
    }

    #[test]
    fn new_singular_candidates() {
        assert!(singular_candidates("policemen").contains(&"policeman".to_string()));
        assert!(singular_candidates("mitoses").contains(&"mitosis".to_string()));
        assert!(singular_candidates("men").is_empty());
        assert!(singular_candidates("ses").is_empty());
    }

    #[test]
    fn singular_candidates_in_rule_order() {
        assert_eq!(
            singular_candidates("cookies"),
            ["cookie", "cooky", "cookie"]
        );
        assert_eq!(singular_candidates("boxes"), ["box", "boxe"]);
        assert_eq!(singular_candidates("wolves"), ["wolve", "wolf", "wolfe"]);
        assert_eq!(singular_candidates("heroes"), ["heroe", "hero"]);
        assert!(singular_candidates("glass").is_empty());
        assert!(singular_candidates("os").is_empty());
        assert!(singular_candidates("pas").is_empty());
    }

    #[test]
    fn checked_singular_takes_first_noun() {
        let nouns = ["cooky", "wolf"];
        assert_eq!(
            checked_singular("cookies", |w| nouns.contains(&w), |_| false).as_deref(),
            Some("cooky")
        );
        assert_eq!(
            checked_singular("wolves", |w| nouns.contains(&w), |_| false).as_deref(),
            Some("wolf")
        );
        assert_eq!(
            checked_singular("cats", |w| nouns.contains(&w), |_| false),
            None
        );
    }

    type Stems = fn(&str, &mut dyn FnMut(&str));

    fn stems(f: Stems, word: &str) -> Vec<String> {
        let mut out = Vec::new();
        f(word, &mut |s| out.push(s.to_string()));
        out
    }

    #[test]
    fn verb_stems_in_rule_order() {
        let v = |w: &str| stems(verb_stems, w);
        assert_eq!(v("carries"), ["carry", "carrie", "carri", "carrie"]);
        assert_eq!(v("carried"), ["carry", "carrie", "carri"]);
        assert_eq!(v("hates"), ["hate", "hat", "hate"]);
        assert_eq!(v("watches"), ["watche", "watch", "watche"]);
        assert_eq!(v("runs"), ["run"]);
        assert_eq!(v("hated"), ["hate", "hat"]);
        assert_eq!(v("wanted"), ["wante", "want"]);
        assert_eq!(v("hating"), ["hate", "hat"]);
        assert_eq!(v("dying"), ["dye", "dy", "die"]);
    }

    #[test]
    fn verb_stems_undouble_after_the_bare_stem() {
        let v = |w: &str| stems(verb_stems, w);
        assert_eq!(v("stopped"), ["stoppe", "stopp", "stop"]);
        assert_eq!(v("running"), ["runne", "runn", "run"]);
        assert_eq!(v("babysitting"), ["babysitte", "babysitt", "babysit"]);
        // an extra candidate only matters if it is a verb: dictionary-checked
        assert_eq!(v("dressed"), ["dresse", "dress", "dres"]);
        assert!(v("backlogged").contains(&"backlog".to_string()));
    }

    #[test]
    fn stems_need_three_characters_ascii_and_a_short_word() {
        // verbs keep two-letter candidates (going to go, being to be) ...
        assert_eq!(stems(verb_stems, "going"), ["goe", "go"]);
        assert_eq!(stems(verb_stems, "being"), ["bee", "be"]);
        assert_eq!(stems(verb_stems, "does"), ["doe", "do", "doe"]);
        assert!(stems(verb_stems, "as").is_empty());
        // ... adjectives need three
        assert_eq!(stems(adjective_stems, "goer"), ["goe"]);
        assert_eq!(stems(adjective_stems, "tiler"), ["tile", "til"]);
        assert!(stems(verb_stems, "caf\u{e9}s").is_empty());
        assert!(stems(verb_stems, &format!("{}ing", "a".repeat(62))).is_empty());
        assert!(!stems(verb_stems, &format!("{}ing", "a".repeat(61))).is_empty());
        assert!(stems(adjective_stems, "caf\u{e9}r").is_empty());
    }

    #[test]
    fn e_restoring_rules_need_a_two_letter_stem() {
        // no "be" from a one-letter stem ...
        assert!(stems(verb_stems, "bed").is_empty());
        assert!(stems(verb_stems, "bing").is_empty());
        assert!(stems(adjective_stems, "ber").is_empty());
        assert!(stems(adjective_stems, "best").is_empty());
        // ... while the bare-stem and -ying rules still reach short lemmas
        assert!(stems(verb_stems, "being").contains(&"be".to_string()));
        assert!(stems(verb_stems, "going").contains(&"go".to_string()));
        assert!(stems(verb_stems, "doing").contains(&"do".to_string()));
        assert!(stems(verb_stems, "does").contains(&"do".to_string()));
        for (w, lemma) in [("dying", "die"), ("tying", "tie"), ("vying", "vie")] {
            assert!(stems(verb_stems, w).contains(&lemma.to_string()), "{w}");
        }
        // a two-letter stem still restores its e
        assert!(stems(verb_stems, "dued").contains(&"due".to_string()));
        assert!(stems(adjective_stems, "bier").contains(&"bie".to_string()));
    }

    #[test]
    fn adjective_stems_in_rule_order() {
        let a = |w: &str| stems(adjective_stems, w);
        assert_eq!(a("happier"), ["happy", "happie", "happi"]);
        assert_eq!(a("happiest"), ["happy", "happie", "happi"]);
        assert_eq!(a("later"), ["late", "lat"]);
        assert_eq!(a("latest"), ["late", "lat"]);
        assert_eq!(a("taller"), ["talle", "tall", "tal"]);
        assert_eq!(a("bigger"), ["bigge", "bigg", "big"]);
        assert_eq!(a("biggest"), ["bigge", "bigg", "big"]);
        assert!(a("cat").is_empty());
    }
}
