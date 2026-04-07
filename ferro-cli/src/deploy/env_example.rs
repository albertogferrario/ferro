//! `.env.example` parser. Returns ordered (key, value) entries with blank
//! and comment lines skipped. Values are kept verbatim (quotes preserved).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
}

/// Parse a `.env`-style string. See module docs and plan 122-02 interfaces.
///
/// Rules:
/// - Skip blank lines and lines whose first non-whitespace char is `#`.
/// - Split each remaining line on the FIRST `=`. Left = key (trimmed),
///   right = value (trimmed, kept verbatim including any surrounding quotes).
/// - Lines without `=` are silently skipped.
/// - Preserves original order; duplicates kept as-is.
pub fn parse_env_example(content: &str) -> Vec<EnvEntry> {
    let mut out = Vec::new();
    for raw in content.lines() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = raw.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        if key.is_empty() {
            continue;
        }
        out.push(EnvEntry {
            key,
            value: value.trim().to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(k: &str, v: &str) -> EnvEntry {
        EnvEntry {
            key: k.to_string(),
            value: v.to_string(),
        }
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let got = parse_env_example("# comment\nFOO=bar\n\nBAZ=qux");
        assert_eq!(got, vec![pair("FOO", "bar"), pair("BAZ", "qux")]);
    }

    #[test]
    fn preserves_quotes_verbatim() {
        let got = parse_env_example("KEY=\"quoted value\"");
        assert_eq!(got, vec![pair("KEY", "\"quoted value\"")]);
    }

    #[test]
    fn splits_on_first_equals_only() {
        let got = parse_env_example("URL=https://x.y/z?a=1");
        assert_eq!(got, vec![pair("URL", "https://x.y/z?a=1")]);
    }

    #[test]
    fn skips_lines_without_equals() {
        let got = parse_env_example("MALFORMED_NO_EQUALS");
        assert!(got.is_empty());
    }

    #[test]
    fn trims_key_and_value() {
        let got = parse_env_example("  SPACED  =  value  ");
        assert_eq!(got, vec![pair("SPACED", "value")]);
    }
}
