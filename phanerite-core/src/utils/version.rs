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
    if is_minecraft_snapshot(&lower) {
        return false;
    }

    true
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
