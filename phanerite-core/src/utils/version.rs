use std::cmp::Ordering;

// 比较两个版本号字符串大小，不严格保证正确，仅用于人类可读信息
/// Compares two version strings; not strictly guaranteed to be correct, for
/// human-readable information only
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let a = tokenize(a);
    let b = tokenize(b);

    for (x, y) in a.iter().zip(b.iter()) {
        let ord = compare_token(x, y);

        if ord != Ordering::Equal {
            return ord;
        }
    }

    // 一个版本结束：
    // 1.0.0 > 1.0.0-beta
    if a.len() != b.len() {
        return match (a.get(b.len()), b.get(a.len())) {
            (Some(Token::Text(_)), None) => Ordering::Less,

            (None, Some(Token::Text(_))) => Ordering::Greater,

            _ => a.len().cmp(&b.len()),
        };
    }

    Ordering::Equal
}

// 判断版本号字符串是否为稳定版，不严格保证正确，仅用于人类可读信息
/// Returns whether a version string denotes a stable release; not strictly
/// guaranteed to be correct, for human-readable information only
pub fn is_stable(version: &str) -> bool {
    let lower = version.to_ascii_lowercase();

    // 明确的不稳定标记
    const UNSTABLE_MARKERS: &[&str] = &[
        "alpha",
        "beta",
        "preview",
        "pre",
        "rc",
        "releasecandidate",
        "snapshot",
        "nightly",
        "dev",
        "devel",
        "development",
        "unstable",
        "exp",
        "experimental",
        "canary",
        "edge",
    ];

    // 处理 -alpha, .beta, _rc 这种
    let normalized = lower.replace(['_', '.'], "-");

    for marker in UNSTABLE_MARKERS {
        for part in normalized.split('-') {
            if part == *marker {
                return false;
            }

            // rc1 beta2 alpha3
            if part.starts_with(marker) && part[marker.len()..].chars().all(|c| c.is_ascii_digit())
            {
                return false;
            }
        }
    }

    // Minecraft snapshot:
    // 23w31a
    // 24w03b
    !is_minecraft_snapshot(&lower) && !is_legacy_minecraft_version(&lower)
}

fn is_legacy_minecraft_version(version: &str) -> bool {
    // Dated pre-release families: Pre-Classic, Indev, and Infdev.
    if ["rd-", "in-", "inf-"].iter().any(|prefix| {
        version
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.as_bytes().first().is_some_and(u8::is_ascii_digit))
    }) {
        return true;
    }

    // Alpha, Beta, and Classic IDs such as a1.2.6, b1.7.3, and c0.30_01c.
    if version
        .strip_prefix(['a', 'b', 'c'])
        .is_some_and(|suffix| suffix.as_bytes().first().is_some_and(u8::is_ascii_digit))
    {
        return true;
    }

    // Some Classic IDs omit the leading c: 0.0.13a_03, for example.
    // A numeric 0.x release alone is not evidence of instability.
    version.starts_with("0.")
        && version.split(['.', '_']).any(|part| {
            let suffix = part.trim_start_matches(|c: char| c.is_ascii_digit());
            suffix.len() < part.len() && matches!(suffix, "a" | "b" | "c")
        })
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Number(u64),
    Text(&'a str),
}

fn tokenize(s: &str) -> Vec<Token<'_>> {
    let mut result = Vec::new();

    let mut start = 0;
    let mut is_num = None;

    for (i, c) in s.char_indices() {
        let current_num = c.is_ascii_digit();

        match is_num {
            None => {
                is_num = Some(current_num);
                start = i;
            }

            Some(last) if last != current_num => {
                if start != i {
                    push_token(&mut result, &s[start..i], last);
                }

                start = i;
                is_num = Some(current_num);
            }

            _ => {}
        }
    }

    if start < s.len() {
        push_token(&mut result, &s[start..], is_num.unwrap_or(false));
    }

    result
}

fn push_token<'a>(out: &mut Vec<Token<'a>>, s: &'a str, num: bool) {
    if num && let Ok(v) = s.parse::<u64>() {
        out.push(Token::Number(v));
        return;
    }
    out.push(Token::Text(s));
}

fn text_weight(s: &str) -> i32 {
    match s.to_ascii_lowercase().as_str() {
        "alpha" | "a" => -40,
        "beta" | "b" => -30,
        "preview" | "pre" => -20,
        "rc" => -10,

        // 常见 build 后缀
        "snapshot" | "dev" | "nightly" => -50,

        _ => 0,
    }
}

fn cmp_text(a: &str, b: &str) -> Ordering {
    let wa = text_weight(a);
    let wb = text_weight(b);

    match wa.cmp(&wb) {
        Ordering::Equal => a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()),

        x => x,
    }
}

fn compare_token(a: &Token<'_>, b: &Token<'_>) -> Ordering {
    match (a, b) {
        (Token::Number(a), Token::Number(b)) => a.cmp(b),

        (Token::Number(_), Token::Text(_)) => {
            // 数字版本通常比后缀大
            Ordering::Greater
        }

        (Token::Text(_), Token::Number(_)) => Ordering::Less,

        (Token::Text(a), Token::Text(b)) => cmp_text(a, b),
    }
}

fn is_minecraft_snapshot(s: &str) -> bool {
    let bytes = s.as_bytes();

    // YYwWWx
    // 例如 23w31a

    if bytes.len() < 5 {
        return false;
    }

    bytes[0..2].iter().all(|x| x.is_ascii_digit())
        && bytes[2] == b'w'
        && bytes[3..5].iter().all(|x| x.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `is_stable` function with various version strings.
    ///
    /// versions are real Minecraft version numbers
    #[test]
    fn test_is_stable() {
        assert!(is_stable("1.7.10"));
        assert!(is_stable("1.19.4"));

        // New era Minecraft stable version that does not start with 1
        assert!(is_stable("26.2"));

        // Release candidate whose "rc" and number are separated by a dash
        assert!(!is_stable("26.3-rc-2"));

        // Release candidate whose "rc" and number are not separated by a dash
        assert!(!is_stable("1.21.5-rc1"));

        // April Fools
        assert!(!is_stable("25w14craftmine"));

        // the only Infdev version that can be downloaded in the Launcher
        assert!(!is_stable("inf-20100618"));

        // Classic version published back in 2009
        assert!(!is_stable("0.0.13a_03"));

        // Pre-release
        assert!(!is_stable("1.21.11-pre5"));

        // Snapshot
        assert!(!is_stable("26.1-snapshot-11"));

        // Experimental
        assert!(!is_stable("1.19-exp1"));
    }

    #[test]
    fn legacy_families_and_experimental_markers_are_unstable() {
        for version in [
            "rd-132211",
            "in-20100223",
            "inf-20100618",
            "a1.2.6",
            "b1.7.3",
            "c0.30_01c",
            "0.0.11a",
            "0.0.13a_03",
            "1.19-exp1",
            "1.19-EXP1",
        ] {
            assert!(!is_stable(version), "{version} is not a stable release");
        }
    }

    #[test]
    fn numeric_loader_and_generic_versions_remain_stable() {
        for version in ["21.1.84", "52.0.28", "0.1.0"] {
            assert!(is_stable(version), "{version} has no unstable marker");
        }
    }

    #[test]
    fn marker_prefixes_do_not_match_unrelated_words() {
        for version in ["1.0-pretty", "1.0-expedition"] {
            assert!(is_stable(version), "{version} has no unstable marker");
        }
    }
}
