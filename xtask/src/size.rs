//! `cargo xtask size`: what the moult crate costs a consumer, in binary
//! bytes, compile time, source bytes and packaged bytes. Every probe is built
//! in a throwaway crate under `target/size-probe` so the workspace is untouched.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

const WITH_MAIN: &str = r#"fn main() {
    let word = std::env::args().nth(1).unwrap_or_else(|| "cities".to_string());
    println!("{}", moult::singularize(&word));
    println!("{}", moult::pluralize(&word));
}
"#;

const WITH_LEMMAS_MAIN: &str = r#"fn main() {
    let word = std::env::args().nth(1).unwrap_or_else(|| "cities".to_string());
    println!("{}", moult::singularize(&word));
    println!("{}", moult::pluralize(&word));
    println!("{}", moult::lemmatize(&word, moult::PartOfSpeech::Verb));
    println!("{}", moult::lemmas(&word).len());
}
"#;

const WITHOUT_MAIN: &str = r#"fn main() {
    let word = std::env::args().nth(1).unwrap_or_else(|| "cities".to_string());
    println!("{word}");
    println!("{word}");
}
"#;

/// Runs `cargo <args>` in `dir`, optionally with an explicit target dir.
fn cargo(dir: &Path, target: Option<&Path>, args: &[&str]) -> Result<Output, String> {
    let program = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(program);
    cmd.current_dir(dir).args(args);
    if let Some(t) = target {
        cmd.env("CARGO_TARGET_DIR", t);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run cargo {}: {e}", args.join(" ")))?;
    if out.status.success() {
        Ok(out)
    } else {
        Err(format!(
            "cargo {} failed in {}:\n{}",
            args.join(" "),
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

fn file_len(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("cannot stat {}: {e}", path.display()))
}

/// Recursive byte total of all regular files under `dir`.
fn tree_len(dir: &Path) -> Result<u64, String> {
    let mut total = 0;
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        total += if path.is_dir() {
            tree_len(&path)?
        } else {
            file_len(&path)?
        };
    }
    Ok(total)
}

/// Writes a throwaway release binary crate and returns its stripped size.
/// `moult_dep` is the absolute path of the moult crate, or `None` for the
/// baseline that does not depend on moult at all.
fn probe_binary(
    base: &Path,
    name: &str,
    main_rs: &str,
    moult_dep: Option<&Path>,
    target: &Path,
) -> Result<u64, String> {
    let dir = base.join(name);
    fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
    let deps = match moult_dep {
        Some(p) => format!(
            "\n[dependencies]\nmoult = {{ path = \"{}\" }}\n",
            p.display()
        ),
        None => String::new(),
    };
    // The empty `[workspace]` table makes each probe its own workspace root,
    // so cargo does not look for the moult workspace above target/.
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n{deps}\n[profile.release]\nstrip = true\n\n[workspace]\n"
    );
    fs::write(dir.join("Cargo.toml"), manifest).map_err(|e| e.to_string())?;
    fs::write(dir.join("src/main.rs"), main_rs).map_err(|e| e.to_string())?;
    cargo(&dir, Some(target), &["build", "--release"])?;
    file_len(&target.join("release").join(name))
}

/// Clean `cargo build -p moult` into `target`, returning wall seconds.
fn clean_build_secs(root: &Path, target: &Path, release: bool) -> Result<f64, String> {
    if target.exists() {
        fs::remove_dir_all(target)
            .map_err(|e| format!("cannot clean {}: {e}", target.display()))?;
    }
    let mut args = vec!["build", "-p", "moult"];
    if release {
        args.push("--release");
    }
    let start = Instant::now();
    cargo(root, Some(target), &args)?;
    Ok(start.elapsed().as_secs_f64())
}

fn mb(bytes: u64) -> f64 {
    bytes as f64 / 1_000_000.0
}

pub fn run() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .map_err(|e| format!("cannot resolve workspace root: {e}"))?;
    let probes = root.join("target/size-probe");
    fs::create_dir_all(&probes).map_err(|e| e.to_string())?;

    // 1. Binary cost of the two probes.
    let bin_target = probes.join("bin-target");
    let with = probe_binary(&probes, "with", WITH_MAIN, Some(&root), &bin_target)?;
    let without = probe_binary(&probes, "without", WITHOUT_MAIN, None, &bin_target)?;
    let delta = with as i64 - without as i64;
    let with_lemmas = probe_binary(
        &probes,
        "with-lemmas",
        WITH_LEMMAS_MAIN,
        Some(&root),
        &bin_target,
    )?;
    let lemma_delta = with_lemmas as i64 - without as i64;

    // 2. Clean compile times.
    let build_target = probes.join("build");
    let debug_secs = clean_build_secs(&root, &build_target, false)?;
    let release_secs = clean_build_secs(&root, &build_target, true)?;

    // 3. Source sizes.
    let src = root.join("src");
    let nouns = file_len(&src.join("nouns.txt"))?;
    let names = file_len(&src.join("names.txt"))?;
    let others = file_len(&src.join("others.txt"))?;
    let data = file_len(&src.join("data.rs"))?;
    let src_total = tree_len(&src)?;

    // 4. Packaged size. Honours CARGO_TARGET_DIR, which cargo package writes to.
    cargo(
        &root,
        None,
        &["package", "-p", "moult", "--allow-dirty", "--no-verify"],
    )?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|l| l.split('"').next())
        .ok_or("no version in Cargo.toml")?
        .to_string();
    let crate_name = format!("moult-{version}.crate");
    let crate_file = target_dir.join("package").join(&crate_name);
    let package = file_len(&crate_file)?;

    println!("moult size report (MB = 1,000,000 bytes)");
    println!();
    println!("binary: release, strip = true, singularize + pluralize");
    println!("  with probe      {with:>12} bytes  {:>7.2} MB", mb(with));
    println!(
        "  without probe   {without:>12} bytes  {:>7.2} MB",
        mb(without)
    );
    println!(
        "  delta           {delta:>12} bytes  {:>7.2} MB",
        delta as f64 / 1_000_000.0
    );
    println!(
        "  with lemmas     {with_lemmas:>12} bytes  {:>7.2} MB",
        mb(with_lemmas)
    );
    println!(
        "  lemma delta     {lemma_delta:>12} bytes  {:>7.2} MB  (singularize + pluralize + lemmatize + lemmas)",
        lemma_delta as f64 / 1_000_000.0
    );
    println!();
    println!("compile time: clean `cargo build -p moult`");
    println!("  debug           {debug_secs:>10.1} s");
    println!("  release         {release_secs:>10.1} s");
    println!();
    println!("source");
    println!("  src/nouns.txt   {nouns:>12} bytes");
    println!("  src/names.txt   {names:>12} bytes");
    println!("  src/others.txt  {others:>12} bytes");
    println!("  src/data.rs     {data:>12} bytes");
    println!("  src/ total      {src_total:>12} bytes");
    println!();
    println!("package: `cargo package -p moult --allow-dirty --no-verify`");
    println!(
        "  {crate_name}  {package:>8} bytes  {:>7.2} MB",
        mb(package)
    );
    Ok(())
}
