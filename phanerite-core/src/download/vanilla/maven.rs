use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Debug)]
pub struct MavenArtifact {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: String,
}

impl MavenArtifact {
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

    pub fn url(&self, base: &Url) -> crate::error::Result<Url> {
        base.join(&self.path())
            .map_err(|e| crate::error::Error::other(e.to_string()))
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
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
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

use std::fmt;

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
