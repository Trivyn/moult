//! The acceptance cases: words that hand-written suffix rules get wrong in
//! one direction or the other. Ignored until the skeleton is implemented.
//! Words with two right answers ("leaves": leaf or leave; "axes": axis or
//! axe) are left out until the crate has a policy for them.

const SINGULAR: &[(&str, &str)] = &[
    // Regular plurals.
    ("territories", "territory"),
    ("boxes", "box"),
    ("glasses", "glass"),
    ("quizzes", "quiz"),
    ("metrics", "metric"),
    ("statuses", "status"),
    ("buses", "bus"),
    ("heroes", "hero"),
    ("potatoes", "potato"),
    ("toes", "toe"),
    ("wolves", "wolf"),
    ("knives", "knife"),
    // "-ies" after "ie" stems, and "-ves" that is not "-f"/"-fe".
    ("cookies", "cookie"),
    ("zombies", "zombie"),
    ("calories", "calorie"),
    ("waves", "wave"),
    ("caves", "cave"),
    ("drives", "drive"),
    ("curves", "curve"),
    // Endings a rule mistakes for an irregular plural.
    ("police", "police"),
    ("slices", "slice"),
    ("quotas", "quota"),
    ("quota", "quota"),
    ("deltas", "delta"),
    // Irregular forms.
    ("mice", "mouse"),
    ("people", "person"),
    ("criteria", "criterion"),
    ("indices", "index"),
    ("matrices", "matrix"),
    ("crises", "crisis"),
    ("analyses", "analysis"),
    // Already singular, though they end in "s".
    ("gas", "gas"),
    ("lens", "lens"),
    ("bus", "bus"),
    ("analysis", "analysis"),
    ("status", "status"),
    // Invariant and mass nouns.
    ("data", "data"),
    ("news", "news"),
    ("series", "series"),
    ("species", "species"),
    ("sheep", "sheep"),
    ("logistics", "logistics"),
    ("physics", "physics"),
    // Case is kept.
    ("Territories", "Territory"),
    ("ISBN", "ISBN"),
    ("ISBNs", "ISBN"),
    ("IDs", "ID"),
];

const PLURAL: &[(&str, &str)] = &[
    ("territory", "territories"),
    ("box", "boxes"),
    ("bus", "buses"),
    ("status", "statuses"),
    ("quiz", "quizzes"),
    ("hero", "heroes"),
    ("potato", "potatoes"),
    ("photo", "photos"),
    ("wolf", "wolves"),
    ("roof", "roofs"),
    ("chief", "chiefs"),
    ("person", "people"),
    ("mouse", "mice"),
    ("criterion", "criteria"),
    ("analysis", "analyses"),
    ("sheep", "sheep"),
    ("data", "data"),
    ("ISBN", "ISBNs"),
    ("Territory", "Territories"),
];

fn check(cases: &[(&str, &str)], f: fn(&str) -> String) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|&(input, want)| {
            let got = f(input);
            (got != want).then(|| format!("{input} -> {got} (want {want})"))
        })
        .collect();
    assert!(wrong.is_empty(), "wrong:\n{}", wrong.join("\n"));
}

#[test]
#[ignore = "skeleton: singularize is not implemented"]
fn singularize_cases() {
    check(SINGULAR, moult::singularize);
}

#[test]
#[ignore = "skeleton: pluralize is not implemented"]
fn pluralize_cases() {
    check(PLURAL, moult::pluralize);
}
