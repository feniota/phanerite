use crate::error::{Error, Result};
use crate::storage::Storage;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 解压缓冲（新线程栈上）
const CHUNK_SIZE: usize = 64 * 1024;
/// 线程栈 = 4× 缓冲，给解压库内部留足空间
const STACK_SIZE: usize = CHUNK_SIZE * 4; // 256 KiB

pub struct Missing;

pub struct ExtractTask {
    path: PathBuf,
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
    pub fn builder() -> ExtractTaskBuilder<Missing> {
        ExtractTaskBuilder {
            path: Missing,
            auto_flattens: false,
            exclude: vec![],
        }
    }

    pub(super) async fn exec(
        &self,
        archive_file: impl AsRef<Path>,
        bucket: Option<PathBuf>,
        storage: &Storage,
    ) -> Result<()> {
        let archive_path = archive_file.as_ref().to_path_buf();
        let target = self.path.clone();
        let auto_flattens = self.auto_flattens;
        let exclude = self.exclude.clone();
        let linker = storage.linker();

        let (tx, rx) = async_channel::bounded(1);
        let _ = std::thread::Builder::new()
            .stack_size(STACK_SIZE)
            .name("extractor".to_string())
            .spawn(move || {
                let result = extract(
                    &archive_path,
                    &target,
                    bucket,
                    auto_flattens,
                    &exclude,
                    &linker,
                );
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

// ── Extraction dispatcher ─────────────────────────────────────────────

fn extract(
    archive: &Path,
    target: &Path,
    bucket: Option<PathBuf>,
    auto_flattens: bool,
    exclude: &[ExcludePattern],
    linker: &impl Fn(&Path, &Path) -> Result<()>,
) -> Result<()> {
    // Peek magic bytes without allocating a BufReader.
    let mut magic = [0u8; 4];
    fs::File::open(archive)?
        .read_exact(&mut magic)
        .map_err(|_| Error::other("archive too small"))?;

    if &magic[..2] == b"PK" {
        unzip(archive, target, bucket, auto_flattens, exclude, linker)
    } else {
        untar(archive, target, bucket, auto_flattens, exclude, linker)
    }
}

// ── ZIP ───────────────────────────────────────────────────────────────

fn unzip(
    archive: &Path,
    target: &Path,
    bucket: Option<PathBuf>,
    auto_flattens: bool,
    exclude: &[ExcludePattern],
    linker: &impl for<'a, 'b> Fn(&'a Path, &'b Path) -> Result<()>,
) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut reader = zip::ZipArchive::new(file)?;

    let mut entries: Vec<String> = Vec::with_capacity(reader.len());
    for i in 0..reader.len() {
        let entry = reader.by_index(i)?;
        let name = entry.name().to_owned();
        if entry.is_dir() || entry.name_raw().ends_with(b"/") {
            continue;
        }
        if exclude.iter().any(|p| p.matches(&name)) {
            continue;
        }
        entries.push(name);
    }

    let prefix = if auto_flattens {
        common_prefix(entries.iter().map(|n| n.as_str()))
    } else {
        ""
    };

    for name in &entries {
        let stored_path = strip_prefix(name, prefix);
        let mut entry = reader.by_name(name)?;
        let dest = target.join(stored_path);
        extract_entry(&mut entry, &dest, bucket.as_deref(), &linker)?;
    }

    Ok(())
}

// ── TAR (auto-detect compression) ─────────────────────────────────────

fn untar(
    archive_path: &Path,
    target: &Path,
    bucket: Option<PathBuf>,
    auto_flattens: bool,
    exclude: &[ExcludePattern],
    linker: &impl Fn(&Path, &Path) -> Result<()>,
) -> Result<()> {
    fn open_tar(archive: &Path) -> Result<tar::Archive<Box<dyn Read>>> {
        let mut magic = [0u8; 6];
        let len = fs::File::open(archive)?.read(&mut magic).unwrap_or(0);

        let file = fs::File::open(archive)?;
        let decoder: Box<dyn Read> = if len >= 3 && magic[0] == 0x1f && magic[1] == 0x8b {
            Box::new(flate2::read::GzDecoder::new(file))
        } else if len >= 3 && &magic[..3] == b"BZh" {
            Box::new(bzip2::read::BzDecoder::new(file))
        } else if len >= 6 && &magic[..6] == b"\xfd7zXZ\x00" {
            Box::new(xz2::read::XzDecoder::new(file))
        } else if len >= 4 && magic[0] == 0x28 && magic[1] == 0xb5 {
            Box::new(zstd::stream::read::Decoder::new(file)?)
        } else {
            Box::new(file)
        };
        Ok(tar::Archive::new(decoder))
    }

    // ── First pass: collect names for prefix detection ────────────
    let mut names: Vec<String> = Vec::new();
    {
        let mut tar = open_tar(archive_path)?;
        for entry in tar.entries()? {
            let entry = entry?;
            if entry.header().entry_type() != tar::EntryType::Regular {
                continue;
            }
            let name = entry.path()?.to_string_lossy().into_owned();
            if exclude.iter().any(|p| p.matches(&name)) {
                continue;
            }
            names.push(name);
        }
    }

    let prefix = if auto_flattens {
        common_prefix(names.iter().map(|n| n.as_str()))
    } else {
        ""
    };

    // ── Second pass: extract, re-applying exclusions ──────────────
    let mut tar = open_tar(archive_path)?;
    for entry in tar.entries()? {
        let entry = entry?;
        if entry.header().entry_type() != tar::EntryType::Regular {
            continue;
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        if exclude.iter().any(|p| p.matches(&name)) {
            continue;
        }
        let stored_path = strip_prefix(&name, prefix);
        let dest = target.join(stored_path);
        extract_entry(entry, &dest, bucket.as_deref(), &linker)?;
    }

    Ok(())
}

// ── Extract a single entry (shared by zip / tar) ─────────────────────

fn extract_entry(
    mut reader: impl Read,
    dest: &Path,
    bucket: Option<&Path>,
    linker: &impl Fn(&Path, &Path) -> Result<()>,
) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut hasher = blake3::Hasher::new();
    let mut chunk = [0u8; CHUNK_SIZE];

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
            let obj_path = bucket_root.join(&hash_str[..2]).join(hash_str);
            if !obj_path.exists() {
                fs::create_dir_all(obj_path.parent().unwrap())?;
                fs::rename(&tmp, &obj_path)?;
            } else {
                let _ = fs::remove_file(&tmp);
            }
            let _ = fs::remove_file(dest);
            linker(&obj_path, dest)?;
        }
        None => {
            let _ = fs::remove_file(dest);
            if fs::rename(&tmp, dest).is_err() {
                let _ = fs::remove_file(&tmp);
            }
        }
    }

    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────

fn strip_prefix<'a>(path: &'a str, prefix: &str) -> &'a str {
    if prefix.is_empty() {
        return path;
    }
    let prefix = prefix.trim_end_matches('/');
    path.strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(path)
}

fn common_prefix<'a>(paths: impl Iterator<Item = &'a str>) -> &'a str {
    let mut prefix: Option<&str> = None;
    for p in paths {
        if p.is_empty() {
            return "";
        }
        match prefix {
            None => prefix = Some(p),
            Some(current) => {
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

fn temp_path(target: &Path) -> PathBuf {
    let mut t = target.as_os_str().to_os_string();
    t.push(".phanerite-tmp");
    PathBuf::from(t)
}

// ── Builder ────────────────────────────────────────────────────────────

pub struct ExtractTaskBuilder<P> {
    path: P,
    auto_flattens: bool,
    exclude: Vec<ExcludePattern>,
}

impl<P> ExtractTaskBuilder<P> {
    pub fn target(self, path: impl Into<PathBuf>) -> ExtractTaskBuilder<PathBuf> {
        ExtractTaskBuilder {
            path: path.into(),
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

impl ExtractTaskBuilder<PathBuf> {
    pub fn build(self) -> ExtractTask {
        ExtractTask {
            path: self.path,
            auto_flattens: self.auto_flattens,
            exclude: self.exclude,
        }
    }
}
