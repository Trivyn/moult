//! Download the pinned release into the cache and verify it.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

/// The lower-case hex SHA-256 of the file at `path`.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Refuse unless the file's SHA-256 equals `expected`.
pub fn verify(path: &Path, expected: &str) -> Result<(), String> {
    let actual = sha256_file(path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "sha256 mismatch for {}\n  expected {expected}\n  actual   {actual}\nrefusing to continue; delete the file or fix data/MANIFEST.toml deliberately",
            path.display()
        ))
    }
}

fn download(url: &str, dest: &Path) -> Result<(), String> {
    eprintln!("downloading {url}");
    let response = ureq::get(url).call().map_err(|e| format!("{url}: {e}"))?;
    let mut reader = response.into_body().into_reader();
    let mut file = File::create(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    io::copy(&mut reader, &mut file).map_err(|e| format!("{url}: {e}"))?;
    Ok(())
}

/// The verified cached copy of the pinned release, downloading it first when
/// absent. The download goes to a temporary file and is renamed into place
/// only after the checksum matches.
pub fn ensure(manifest: &Manifest, cache_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(cache_dir).map_err(|e| format!("{}: {e}", cache_dir.display()))?;
    let path = cache_dir.join(manifest.basename());
    if !path.exists() {
        let tmp = cache_dir.join(format!("{}.part", manifest.basename()));
        download(&manifest.url, &tmp)?;
        if let Err(e) = verify(&tmp, &manifest.sha256) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    verify(&path, &manifest.sha256)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_accepts_match_and_refuses_mismatch() {
        let path = std::env::temp_dir().join("moult-xtask-verify-test");
        fs::write(&path, b"abc").unwrap();
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify(&path, abc).is_ok());
        let err = verify(&path, &"0".repeat(64)).unwrap_err();
        assert!(err.contains("mismatch") && err.contains(abc));
        fs::remove_file(&path).unwrap();
    }
}
