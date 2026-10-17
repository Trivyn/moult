//! Dev-only tooling for moult. Run as `cargo xtask <command>`; never
//! published and never a dependency of the runtime crate.

extern crate alloc;

#[path = "../../src/policy.rs"]
mod policy;
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

const USAGE: &str = "usage: cargo xtask report\n       cargo xtask gen-data [--check]\n       cargo xtask size\n       cargo xtask-inflect [--guess] < words.txt   (alias for `run -p xtask --features inflect -- inflect`)";

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
        .sum::<usize>();
    if unclassified > 0 || !an.problems.is_empty() {
        return Err(format!(
            "refusing to generate: {unclassified} unclassified and {} stale or invalid policy entries; run `cargo xtask report` and fix src/policy.rs",
            an.problems.len()
        ));
    }
    let out = gen::render(&manifest, &wn);
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
        Some("inflect") if args[1..].iter().all(|f| f == "--guess") => {
            inflect::run(args.len() > 1)
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
}
