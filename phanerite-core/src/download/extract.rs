use crate::error::{Error, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CHUNK_SIZE: usize = 8192;

#[derive(Clone, Copy)]
pub enum ArchiveFormat {
    Zip,
    Tar,
}

pub struct Missing;

pub struct ExtractTask {
    path: PathBuf,
    format: ArchiveFormat,
    auto_flattens: bool,
    exclude: Vec<ExcludePattern>,
}

/// A simple exclusion pattern for archive entries.
/// Mojang convention: `META-INF/` = prefix, `*.SF` = suffix.
#[derive(Clone)]
pub enum ExcludePattern {
    Prefix(String),
    Suffix(String),
}

impl ExcludePattern {
    fn matches(&self, path: &str) -> bool {
        match self {
            ExcludePattern::Prefix(p) => path.starts_with(p.as_str()),
            ExcludePattern::Suffix(p) => path.ends_with(p.as_str()),
        }
    }
}

impl ExtractTask {
    pub fn builder() -> ExtractTaskBuilder<Missing, Missing> {
        ExtractTaskBuilder {
            path: Missing,
            format: Missing,
            auto_flattens: false,
            exclude: vec![],
        }
    }

    pub(super) async fn exec(
        &self,
        archive_file: impl AsRef<Path>,
        bucket: Option<PathBuf>,
        _buf: &mut [u8],
    ) -> Result<()> {
        let archive_path = archive_file.as_ref().to_path_buf();
        let target = self.path.clone();
        let format = self.format;
        let auto_flattens = self.auto_flattens;
        let exclude = self.exclude.clone();

        let (tx, rx) = async_channel::bounded(1);
        std::thread::spawn(move || {
            let result = match format {
                ArchiveFormat::Zip => {
                    unzip(&archive_path, &target, bucket, auto_flattens, &exclude)
                }
                ArchiveFormat::Tar => {
                    untar(&archive_path, &target, bucket, auto_flattens, &exclude)
                }
            };
            // Native jars are delivery vehicles — remove after extraction.
            if result.is_ok() {
                let _ = fs::remove_file(&archive_path);
            }
            let _ = tx.send_blocking(result);
        });
        rx.recv()
            .await
            .map_err(|_| Error::other("extract thread panicked"))?
    }
}

// ── ZIP ──────────────────────────────────────────────────────────────

fn unzip(
    archive: &Path,
    target: &Path,
    bucket: Option<PathBuf>,
    auto_flattens: bool,
    exclude: &[ExcludePattern],
) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut reader = zip::ZipArchive::new(file)?;

    // ── List entries ──────────────────────────────────────────────
    let mut entries: Vec<(String, u64)> = Vec::with_capacity(reader.len());
    for i in 0..reader.len() {
        let entry = reader.by_index(i)?;
        let name = entry.name().to_owned();
        if entry.is_dir() || entry.name_raw().ends_with(b"/") {
            continue;
        }
        if exclude.iter().any(|p| p.matches(&name)) {
            continue;
        }
        entries.push((name, entry.size()));
    }

    let prefix = if auto_flattens {
        common_prefix(entries.iter().map(|(n, _)| n.as_str()))
    } else {
        ""
    };

    // ── Extract one by one ────────────────────────────────────────
    for (name, _) in &entries {
        let stored_path = strip_prefix(name, prefix);
        let mut entry = reader.by_name(name)?;
        let dest = target.join(stored_path);
        extract_entry(&mut entry, &dest, bucket.as_deref())?;
    }

    Ok(())
}

// ── TAR ─────────────────────────────────────────────────────────────

fn untar(
    archive_path: &Path,
    target: &Path,
    bucket: Option<PathBuf>,
    auto_flattens: bool,
    exclude: &[ExcludePattern],
) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let mut tar = tar::Archive::new(file);

    // ── List entries ──────────────────────────────────────────────
    let mut entries: Vec<(String, u64)> = Vec::new();
    for entry in tar.entries()? {
        let entry = entry?;
        let header = entry.header();
        if header.entry_type() != tar::EntryType::Regular {
            continue;
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        if exclude.iter().any(|p| p.matches(&name)) {
            continue;
        }
        let size = header.size()?;
        entries.push((name, size));
    }

    let prefix = if auto_flattens {
        common_prefix(entries.iter().map(|(n, _)| n.as_str()))
    } else {
        ""
    };

    // Re-open for extraction
    drop(tar);
    let file = fs::File::open(archive_path)?;
    let mut tar = tar::Archive::new(file);

    for entry in tar.entries()? {
        let entry = entry?;
        let header = entry.header();
        if header.entry_type() != tar::EntryType::Regular {
            continue;
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        if !entries.iter().any(|(n, _)| n == &name) {
            continue;
        }
        let stored_path = strip_prefix(&name, prefix);
        let dest = target.join(stored_path);
        extract_entry(entry, &dest, bucket.as_deref())?;
    }

    Ok(())
}

// ── Extract a single entry (shared by zip / tar) ────────────────────

fn extract_entry(mut reader: impl Read, dest: &Path, bucket: Option<&Path>) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut hasher = blake3::Hasher::new();
    let mut chunk = vec![0u8; CHUNK_SIZE];

    // Write to a temp file so failed extractions don't leave partial junk.
    let tmp = temp_path(dest);
    let mut tmp_file = fs::File::create(&tmp)?;

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let data = &chunk[..n];
        hasher.update(data);
        tmp_file.write_all(data)?;
    }

    let hash = hasher.finalize();
    let hash_hex = hash.to_hex();
    let hash_str = hash_hex.as_str();

    tmp_file.flush()?;
    drop(tmp_file);

    match bucket {
        Some(bucket_root) => {
            // Content-addressed: obj/{first2}/{full_hash}
            let obj_path = bucket_root.join(&hash_str[..2]).join(hash_str);
            if !obj_path.exists() {
                fs::create_dir_all(obj_path.parent().unwrap())?;
                fs::rename(&tmp, &obj_path)?;
            } else {
                let _ = fs::remove_file(&tmp);
            }
            // Hardlink from dest → bucket; fall back to copy if cross-device.
            if fs::hard_link(&obj_path, dest).is_err() {
                fs::copy(&obj_path, dest)?;
            }
        }
        None => {
            fs::rename(&tmp, dest)?;
        }
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Strip the leading `prefix` directory component from `path`.
fn strip_prefix<'a>(path: &'a str, prefix: &str) -> &'a str {
    if prefix.is_empty() {
        return path;
    }
    let prefix = prefix.trim_end_matches('/');
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(path)
}

/// Longest common path prefix among non-empty slices (directory level).
fn common_prefix<'a>(paths: impl Iterator<Item = &'a str>) -> &'a str {
    let mut prefix: Option<&str> = None;
    for p in paths {
        if p.is_empty() {
            return "";
        }
        match prefix {
            None => prefix = Some(p),
            Some(current) => {
                // Shorten `current` until it's a prefix of `p`
                let mut cur = current;
                while !p.starts_with(cur) {
                    cur = match cur.rfind('/') {
                        Some(pos) => &cur[..pos],
                        None => return "",
                    };
                }
                prefix = Some(cur);
            }
        }
    }
    prefix.unwrap_or("")
}

/// Generate a temp path next to `target`.
fn temp_path(target: &Path) -> PathBuf {
    let mut t = target.as_os_str().to_os_string();
    t.push(".phanerite-tmp");
    PathBuf::from(t)
}

// ── Builder ──────────────────────────────────────────────────────────

pub struct ExtractTaskBuilder<P, F> {
    path: P,
    format: F,
    auto_flattens: bool,
    exclude: Vec<ExcludePattern>,
}

impl<P, F> ExtractTaskBuilder<P, F> {
    pub fn target(self, path: impl Into<PathBuf>) -> ExtractTaskBuilder<PathBuf, F> {
        ExtractTaskBuilder {
            path: path.into(),
            format: self.format,
            auto_flattens: self.auto_flattens,
            exclude: self.exclude,
        }
    }
    pub fn zip(self) -> ExtractTaskBuilder<P, ArchiveFormat> {
        ExtractTaskBuilder {
            path: self.path,
            format: ArchiveFormat::Zip,
            auto_flattens: self.auto_flattens,
            exclude: self.exclude,
        }
    }
    pub fn tar(self) -> ExtractTaskBuilder<P, ArchiveFormat> {
        ExtractTaskBuilder {
            path: self.path,
            format: ArchiveFormat::Tar,
            auto_flattens: self.auto_flattens,
            exclude: self.exclude,
        }
    }
    pub fn flatten(mut self) -> Self {
        self.auto_flattens = true;
        self
    }
    pub fn exclude(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for p in patterns {
            let s: String = p.into();
            if let Some(suffix) = s.strip_prefix('*') {
                self.exclude.push(ExcludePattern::Suffix(suffix.to_owned()));
            } else {
                self.exclude.push(ExcludePattern::Prefix(s));
            }
        }
        self
    }
}

impl ExtractTaskBuilder<PathBuf, ArchiveFormat> {
    pub fn build(self) -> ExtractTask {
        ExtractTask {
            path: self.path,
            format: self.format,
            auto_flattens: self.auto_flattens,
            exclude: self.exclude,
        }
    }
}
