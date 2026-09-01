use crate::download::extract::ExtractTask;
use crate::download::task::DownloadTask;
use crate::instance::manifest::{Arch, Artifact, Extract, Library, Os, Rule};
use crate::storage::Storage;
use std::collections::HashSet;
use std::path::Path;

// `natives-*` 分类器里编码的平台
/// The platform encoded in a `natives-*` classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeTarget {
    os: Os,
    // 后缀不是已知架构时为 `None`，视作任意架构，免得误伤第三方分类器
    /// `None` when the suffix is not a known architecture — treated as
    /// "any architecture" so third-party classifiers are not dropped.
    arch: Option<Arch>,
}

impl Library {
    pub fn to_task<'cx>(
        &self,
        storage: &'cx Storage,
        features: &HashSet<&'static str>,
    ) -> Option<DownloadTask<'cx>> {
        if !self.allowed_env(features) {
            return None;
        }
        let a = self.downloads.as_ref()?.artifact.clone()?;
        Some(
            DownloadTask::builder()
                .url(a.url)
                .to_library(a.path, storage)
                .file_name(self.name.clone())
                .file_size(a.size)
                .hash(a.sha1)
                .build(),
        )
    }

    pub fn to_native_task<'cx>(
        &self,
        storage: &'cx Storage,
        features: &HashSet<&'static str>,
        native_dir: impl AsRef<Path>,
    ) -> Option<DownloadTask<'cx>> {
        if !self.allowed_env(features) {
            return None;
        }

        // ── v1: natives map + classifiers ────────────────
        if let Some(natives_map) = &self.natives {
            let classifiers = self.downloads.as_ref()?.classifiers.as_ref()?;
            let classifier_key = natives_map.get(&Os::current()?.to_string())?;
            // Old manifests spell the 32/64-bit split as `natives-windows-${arch}`.
            let classifier_key = classifier_key.replace("${arch}", arch_bits());
            let artifact = classifiers.get(&classifier_key)?;
            return Some(native_download(
                storage,
                artifact,
                native_dir.as_ref(),
                &format!("{}-{}", self.name, classifier_key),
                self.extract.as_ref(),
            ));
        }

        // ── v2: classifier suffix in Maven name ──────────
        // e.g. `org.lwjgl:lwjgl:3.4.1:natives-windows`. `allowed_env` has
        // already matched the classifier against the current platform.
        if self.native_target().is_some() {
            let a = self.downloads.as_ref()?.artifact.clone()?;
            return Some(native_download(
                storage,
                &a,
                native_dir.as_ref(),
                &self.name.to_string(),
                self.extract.as_ref(),
            ));
        }

        None
    }

    fn allowed_env(&self, features: &HashSet<&'static str>) -> bool {
        self.rules_allow(features) && self.classifier_matches_current()
    }

    // 按顺序求值，后面的规则覆盖前面的；带 `rules` 时默认不允许
    /// Evaluates the rules in order, later ones overriding earlier ones.
    /// A library that carries `rules` is denied unless one of them allows it;
    /// no rules at all — or an empty list — means unconditional.
    fn rules_allow(&self, features: &HashSet<&'static str>) -> bool {
        self.rules
            .as_deref()
            .is_none_or(|rules| Rule::allows(rules, features))
    }

    // 1.19 起 natives 库的 rules 只按操作系统区分，架构写在分类器里：Windows 上
    // `natives-windows`、`natives-windows-arm64`、`natives-windows-x86` 的规则
    // 全部通过，所以还得再用分类器筛一次
    /// The natives classifier is the only architecture gate modern manifests
    /// have: from 1.19 on their rules match the OS name alone, so on Windows
    /// `natives-windows`, `natives-windows-arm64` and `natives-windows-x86`
    /// are all allowed by the rules and must be filtered by classifier.
    fn classifier_matches_current(&self) -> bool {
        self.native_target().is_none_or(|target| {
            Os::current() == Some(target.os)
                && target.arch.is_none_or(|arch| Arch::current() == Some(arch))
        })
    }

    /// Parses the platform out of a `natives-*` classifier, e.g.
    /// `natives-windows-arm64` → windows/arm64, `natives-linux` → linux/x86_64.
    ///
    /// Returns `None` when the name carries no natives classifier.
    fn native_target(&self) -> Option<NativeTarget> {
        let rest = self.name.classifier.as_deref()?.strip_prefix("natives-")?;
        let (os, arch) = match rest.split_once('-') {
            Some((os, arch)) => (os, Some(arch)),
            None => (rest, None),
        };
        Some(NativeTarget {
            os: os.parse().ok()?,
            arch: match arch {
                // No suffix means the 64-bit build.
                None => Some(Arch::X64),
                // LWJGL 3.3.3 ships the fixed macOS x64 freetype here.
                Some("patch") => Some(Arch::X64),
                Some(a) => a.parse().ok(),
            },
        })
    }
}

fn arch_bits() -> &'static str {
    if cfg!(target_pointer_width = "64") {
        "64"
    } else {
        "32"
    }
}

fn native_download<'cx>(
    storage: &'cx Storage,
    artifact: &Artifact,
    native_dir: &Path,
    file_name: &str,
    extract: Option<&Extract>,
) -> DownloadTask<'cx> {
    let mut builder = ExtractTask::builder().target(native_dir).flatten();
    if let Some(ex) = extract
        && let Some(ref patterns) = ex.exclude
    {
        builder = builder.exclude(patterns.iter().cloned());
    }
    DownloadTask::builder()
        .url(artifact.url.clone())
        .extract_to(builder.build(), storage)
        .file_name(file_name)
        .file_size(artifact.size)
        .hash(artifact.sha1.clone())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(name: &str, rules: &str) -> Library {
        serde_json::from_str(&format!(r#"{{"name":"{name}"{rules}}}"#)).unwrap()
    }

    /// Every 1.19+ natives variant carries the same OS-only rule, so the
    /// classifier has to do the architecture filtering.
    #[test]
    fn v2_natives_are_gated_by_classifier() {
        let windows = r#","rules":[{"action":"allow","os":{"name":"windows"}}]"#;
        let osx = r#","rules":[{"action":"allow","os":{"name":"osx"}}]"#;
        let cases = [
            (
                "org.lwjgl:lwjgl:3.4.1:natives-windows",
                windows,
                Os::Windows,
                Arch::X64,
            ),
            (
                "org.lwjgl:lwjgl:3.4.1:natives-windows-x86",
                windows,
                Os::Windows,
                Arch::X86,
            ),
            (
                "org.lwjgl:lwjgl:3.4.1:natives-windows-arm64",
                windows,
                Os::Windows,
                Arch::Arm64,
            ),
            (
                "org.lwjgl:lwjgl:3.4.1:natives-macos",
                osx,
                Os::Osx,
                Arch::X64,
            ),
            (
                "org.lwjgl:lwjgl:3.4.1:natives-macos-arm64",
                osx,
                Os::Osx,
                Arch::Arm64,
            ),
            // LWJGL 3.3.3's macOS x64 freetype fix.
            (
                "org.lwjgl:lwjgl-freetype:3.3.3:natives-macos-patch",
                osx,
                Os::Osx,
                Arch::X64,
            ),
        ];

        for (name, rules, os, arch) in cases {
            let lib = library(name, rules);
            assert_eq!(
                lib.native_target(),
                Some(NativeTarget {
                    os,
                    arch: Some(arch)
                }),
                "{name}"
            );
            assert_eq!(
                lib.allowed_env(&HashSet::new()),
                Os::current() == Some(os) && Arch::current() == Some(arch),
                "{name}"
            );
        }
    }

    /// Non-natives classifiers keep their rules-only gate — netty ships both
    /// `linux-x86_64` and `linux-aarch_64` and picks at runtime.
    #[test]
    fn other_classifiers_are_left_to_the_rules() {
        for name in [
            "io.netty:netty-transport-native-epoll:4.2.15.Final:linux-aarch_64",
            "org.lwjgl:lwjgl:3.4.1:unsafe",
            "org.lwjgl:lwjgl:3.4.1",
        ] {
            let lib = library(name, "");
            assert_eq!(lib.native_target(), None, "{name}");
            assert!(lib.allowed_env(&HashSet::new()), "{name}");
        }
    }

    /// An unknown suffix must not silently drop the library.
    #[test]
    fn unknown_native_arch_matches_any_arch() {
        let lib = library("com.example:thing:1:natives-linux-riscv", "");
        assert_eq!(
            lib.native_target(),
            Some(NativeTarget {
                os: Os::Linux,
                arch: None
            })
        );
        assert_eq!(
            lib.allowed_env(&HashSet::new()),
            Os::current() == Some(Os::Linux)
        );
    }

    /// Legacy rules: `allow` then `disallow` for one OS — the last matching
    /// rule wins, and a library with rules is denied by default.
    #[test]
    fn legacy_rules_are_evaluated_in_order() {
        let all_but_osx = library(
            "org.lwjgl.lwjgl:lwjgl-platform:2.9.4",
            r#","rules":[{"action":"allow"},{"action":"disallow","os":{"name":"osx"}}]"#,
        );
        assert_eq!(
            all_but_osx.allowed_env(&HashSet::new()),
            Os::current() != Some(Os::Osx)
        );

        let osx_only = library(
            "org.lwjgl.lwjgl:lwjgl-platform:2.9.2",
            r#","rules":[{"action":"allow","os":{"name":"osx"}}]"#,
        );
        assert_eq!(
            osx_only.allowed_env(&HashSet::new()),
            Os::current() == Some(Os::Osx)
        );
    }

    #[test]
    fn features_gate_rules() {
        let lib = library(
            "com.example:demo:1",
            r#","rules":[{"action":"allow","features":{"is_demo_user":true}}]"#,
        );
        assert!(!lib.allowed_env(&HashSet::new()));
        assert!(lib.allowed_env(&HashSet::from(["is_demo_user"])));
    }
}
