use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use url::Url;

// 用于解析 Maven 坐标，并根据 Maven 坐标生成路径和 URL
/// Parses Maven coordinates and builds paths and URLs from them
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MavenArtifact {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: String,
}

impl MavenArtifact {
    // 生成路径字符串
    /// Builds the path string
    pub fn path(&self) -> String {
        let mut filename = format!("{}-{}", self.artifact, self.version);

        if let Some(c) = &self.classifier {
            filename.push('-');
            filename.push_str(c);
        }

        filename.push('.');
        filename.push_str(&self.extension);

        format!(
            "{}/{}/{}/{}",
            self.group.replace('.', "/"),
            self.artifact,
            self.version,
            filename
        )
    }
    // 生成路径 URL
    /// Builds the path URL
    pub fn url(&self, base: &Url) -> crate::error::Result<Url> {
        base.join(&self.path())
            .map_err(|e| crate::error::Error::other(e.to_string()))
    }

    /// Generates the SHA-1 checksum URL.
    pub fn sha1_url(&self, base: &Url) -> crate::error::Result<Url> {
        self.suffix_url(base, ".sha1")
    }

    /// Generates the MD5 checksum URL.
    pub fn md5_url(&self, base: &Url) -> crate::error::Result<Url> {
        self.suffix_url(base, ".md5")
    }

    /// Generates the SHA-256 checksum URL.
    pub fn sha256_url(&self, base: &Url) -> crate::error::Result<Url> {
        self.suffix_url(base, ".sha256")
    }

    /// Generates the SHA-512 checksum URL.
    pub fn sha512_url(&self, base: &Url) -> crate::error::Result<Url> {
        self.suffix_url(base, ".sha512")
    }

    /// Generates the POM URL for this artifact.
    pub fn pom_url(&self, base: &Url) -> crate::error::Result<Url> {
        let path = format!(
            "{}/{}/{}/{}-{}.pom",
            self.group.replace('.', "/"),
            self.artifact,
            self.version,
            self.artifact,
            self.version,
        );

        base.join(&path)
            .map_err(|e| crate::error::Error::other(e.to_string()))
    }

    /// Generates the Gradle Module Metadata URL for this artifact.
    pub fn module_url(&self, base: &Url) -> crate::error::Result<Url> {
        self.suffix_url(base, ".module")
    }

    fn suffix_url(&self, base: &Url, suffix: &str) -> crate::error::Result<Url> {
        let mut url = self.url(base)?;
        let path = format!("{}{}", url.path(), suffix);

        url.set_path(&path);

        Ok(url)
    }
}

impl FromStr for MavenArtifact {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (coords, extension) = match s.split_once('@') {
            Some((left, ext)) => (left, ext.to_string()),
            None => (s, "jar".to_string()),
        };

        let mut parts = coords.split(':');

        let group = parts.next().ok_or("missing group")?;

        let artifact = parts.next().ok_or("missing artifact")?;

        let version = parts.next().ok_or("missing version")?;

        let classifier = parts.next().map(String::from);

        if parts.next().is_some() {
            return Err("too many components".into());
        }

        Ok(Self {
            group: group.into(),
            artifact: artifact.into(),
            version: version.into(),
            classifier,
            extension,
        })
    }
}

impl<'de> Deserialize<'de> for MavenArtifact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for MavenArtifact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut value = format!("{}:{}:{}", self.group, self.artifact, self.version);

        if let Some(classifier) = &self.classifier {
            value.push(':');
            value.push_str(classifier);
        }

        if self.extension != "jar" {
            value.push('@');
            value.push_str(&self.extension);
        }

        serializer.serialize_str(&value)
    }
}

impl fmt::Display for MavenArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)?;

        if let Some(classifier) = &self.classifier {
            write!(f, ":{}", classifier)?;
        }

        if self.extension != "jar" {
            write!(f, "@{}", self.extension)?;
        }

        Ok(())
    }
}

impl From<MavenArtifact> for String {
    fn from(value: MavenArtifact) -> Self {
        value.to_string()
    }
}
