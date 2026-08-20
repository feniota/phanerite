//! Crash report data, local signature matching, and diagnostic redaction.

use crate::{
    route::{CrashRef, InstanceRef, StorageId},
    state::{Loader, StorageRegistry},
};

/// A local, signature-based match. Rules only report patterns that appear
/// verbatim in the captured output; this is never an AI diagnosis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashFinding {
    pub rule: String,
    pub title: String,
    pub explanation: String,
    pub evidence_lines: Vec<usize>,
    pub implicated_mod_ids: Vec<String>,
    pub suggested_memory: Option<u32>,
}

impl CrashFinding {
    /// Whether the finding offers a one-click retry action.
    pub fn has_action(&self) -> bool {
        !self.implicated_mod_ids.is_empty() || self.suggested_memory.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashSource {
    Aphanite,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashEnvironment {
    pub mc_version: String,
    pub loader: Loader,
    pub loader_version: String,
    pub java_name: String,
    pub java_version: u32,
    pub java_path: String,
    pub memory: u32,
    pub os: String,
    pub gpu: String,
    pub enabled_mods: Vec<(String, String)>,
    pub active_overrides: Vec<String>,
    pub source: CrashSource,
    pub aphanite_server: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashReport {
    pub storage_id: StorageId,
    pub id: String,
    pub instance_id: String,
    pub when: String,
    pub exit_code: i32,
    /// `None` when Minecraft never wrote a crash report.
    pub lines: Option<Vec<String>>,
    pub stderr_tail: Vec<String>,
    pub hs_err_path: Option<String>,
    pub findings: Vec<CrashFinding>,
    pub environment: CrashEnvironment,
}

impl CrashReport {
    pub fn reference(&self) -> CrashRef {
        CrashRef::new(self.storage_id, self.id.clone())
    }

    pub fn instance(&self) -> InstanceRef {
        InstanceRef::new(self.storage_id, self.instance_id.clone())
    }

    pub fn has_report(&self) -> bool {
        self.lines.is_some()
    }

    /// The output panel content: the crash report when present, otherwise the
    /// captured stderr tail.
    pub fn output(&self) -> &[String] {
        match &self.lines {
            Some(lines) => lines,
            None => &self.stderr_tail,
        }
    }

    pub fn source_text(&self) -> String {
        self.output().join("\n")
    }
}

/// Removes credentials and identifying home-directory segments from shareable
/// diagnostics. Mirrors `design/src/lib/redact.ts`.
pub fn redact(text: &str) -> String {
    let mut result = redact_credential_arguments(text);
    result = redact_home_paths(&result);
    redact_jwt_like(&result)
}

fn redact_credential_arguments(text: &str) -> String {
    const KEYS: [&str; 3] = ["--accessToken", "--clientToken", "--session"];
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    'outer: while !rest.is_empty() {
        for key in KEYS {
            if let Some(start) = find_ignore_ascii_case(rest, key) {
                let after_key = start + key.len();
                let tail = &rest[after_key..];
                let spaces = tail.len() - tail.trim_start_matches([' ', '\t']).len();
                if spaces == 0 {
                    continue;
                }
                let value_start = after_key + spaces;
                let value = &rest[value_start..];
                let value_len = quoted_or_bare_len(value);
                if value_len == 0 {
                    continue;
                }
                result.push_str(&rest[..value_start]);
                result.push_str("<redacted>");
                rest = &rest[value_start + value_len..];
                continue 'outer;
            }
        }
        result.push_str(rest);
        break;
    }
    result
}

fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    let lowered = haystack.to_ascii_lowercase();
    lowered.find(&needle.to_ascii_lowercase())
}

/// Length of a `"..."`, `'...'`, or bare non-whitespace argument value.
fn quoted_or_bare_len(value: &str) -> usize {
    let mut characters = value.char_indices();
    let Some((_, first)) = characters.next() else {
        return 0;
    };
    if first == '"' || first == '\'' {
        for (index, character) in characters {
            if character == first {
                return index + character.len_utf8();
            }
        }
        return value.len();
    }
    value.find(char::is_whitespace).unwrap_or(value.len())
}

fn redact_home_paths(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("/home/") {
        result.push_str(&rest[..index]);
        let after = &rest[index + "/home/".len()..];
        let name_len = after
            .find(|character: char| {
                character == '/' || character == '\\' || character.is_whitespace()
            })
            .unwrap_or(after.len());
        let separator = after[name_len..].chars().next();
        if name_len == 0 || !matches!(separator, Some('/') | Some('\\')) {
            result.push_str("/home/");
            rest = after;
            continue;
        }
        result.push_str("~/");
        rest = &after[name_len + 1..];
    }
    result.push_str(rest);
    result
}

/// Replaces `eyJ…`-prefixed three-segment tokens, the shape of a JWT.
fn redact_jwt_like(text: &str) -> String {
    let is_token_char =
        |character: char| character.is_ascii_alphanumeric() || character == '_' || character == '-';
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("eyJ") {
        // Only a token boundary starts a JWT, so `keyJ` is left alone.
        let preceded_by_token_char = rest[..index]
            .chars()
            .next_back()
            .is_some_and(|character| is_token_char(character) || character == '.');
        let candidate = &rest[index..];
        let length = candidate
            .find(|character: char| !(is_token_char(character) || character == '.'))
            .unwrap_or(candidate.len());
        let token = &candidate[..length];
        let segments: Vec<&str> = token.split('.').collect();
        let looks_like_jwt = !preceded_by_token_char
            && segments.len() == 3
            && segments
                .iter()
                .all(|segment| !segment.is_empty() && segment.chars().all(is_token_char));
        result.push_str(&rest[..index]);
        if looks_like_jwt {
            result.push_str("<redacted>");
            rest = &candidate[length..];
        } else {
            result.push_str("eyJ");
            rest = &candidate["eyJ".len()..];
        }
    }
    result.push_str(rest);
    result
}

#[derive(Default)]
pub struct CrashStore {
    reports: Vec<CrashReport>,
    current_storage: Option<StorageId>,
    revision: u64,
}

impl CrashStore {
    pub fn new(reports: Vec<CrashReport>) -> Self {
        Self {
            reports,
            ..Default::default()
        }
    }

    pub fn all(&self) -> &[CrashReport] {
        &self.reports
    }

    pub fn get(&self, storage: StorageId, id: &str) -> Option<&CrashReport> {
        self.reports
            .iter()
            .find(|report| report.storage_id == storage && report.id == id)
    }

    pub fn find(&self, reference: &CrashRef) -> Option<&CrashReport> {
        self.get(reference.storage_id, &reference.report_id)
    }

    pub fn set_storage_context(&mut self, storage: StorageId) {
        self.current_storage = Some(storage);
    }

    pub fn storage_context(&self) -> Option<StorageId> {
        self.current_storage
    }

    /// Drops reports belonging to a deleted instance.
    pub fn remove_instance(&mut self, instance: &InstanceRef) -> bool {
        let before = self.reports.len();
        self.reports.retain(|report| {
            report.storage_id != instance.storage_id || report.instance_id != instance.instance_id
        });
        if self.reports.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn apply_for_storage(
        &mut self,
        registry: &StorageRegistry,
        storage: StorageId,
        reports: Vec<CrashReport>,
    ) -> bool {
        if registry.get(storage).is_none() || self.current_storage != Some(storage) {
            return false;
        }
        if self.reports == reports {
            return false;
        }
        self.reports = reports;
        self.revision += 1;
        true
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

/// The Aphanite server summary shown on the Aphanite page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AphaniteSummary {
    pub storage_id: StorageId,
    pub server_name: String,
    pub server_url: String,
}
