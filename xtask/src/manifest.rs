//! `data/MANIFEST.toml`: the pinned source release.

use std::path::Path;

pub struct Manifest {
    pub name: String,
    pub edition: String,
    pub license: String,
    pub url: String,
    pub sha256: String,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Manifest, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Manifest::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Manifest, String> {
        let table: toml::Table = text.parse().map_err(|e| format!("MANIFEST.toml: {e}"))?;
        let source = table
            .get("source")
            .and_then(|v| v.as_table())
            .ok_or("MANIFEST.toml: missing [source]")?;
        let field = |key: &str| -> Result<String, String> {
            source
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .ok_or_else(|| format!("MANIFEST.toml: [source] is missing `{key}`"))
        };
        Ok(Manifest {
            name: field("name")?,
            edition: field("edition")?,
            license: field("license")?,
            url: field("url")?,
            sha256: field("sha256")?.to_ascii_lowercase(),
        })
    }

    /// The file name of the download: the last path segment of the URL.
    pub fn basename(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_source_table() {
        let m = Manifest::parse(
            "[source]\nname=\"N\"\nedition=\"1\"\nlicense=\"L\"\nurl=\"http://x/y/z.xml.gz\"\nsha256=\"AB\"\n",
        )
        .unwrap();
        assert_eq!(m.basename(), "z.xml.gz");
        assert_eq!(m.sha256, "ab");
        assert!(Manifest::parse("[other]\n").is_err());
    }
}
