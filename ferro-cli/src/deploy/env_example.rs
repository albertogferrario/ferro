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
        let trimmed_val = value.trim();
        let stripped = strip_inline_comment(trimmed_val);
        out.push(EnvEntry {
            key,
            value: stripped.to_string(),
        });
    }
    out
}

/// Strip a trailing ` #comment` from an unquoted value.
///
/// Rules (D-01..D-03):
/// - Values starting with `"` or `'` are returned as-is (D-02).
/// - Otherwise, the first `#` preceded by whitespace terminates the value;
///   everything from that whitespace onward is removed (D-01).
/// - A `#` with no preceding whitespace is treated as part of the value (D-03).
fn strip_inline_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.first() == Some(&b'"') || bytes.first() == Some(&b'\'') {
        return value;
    }
    let mut prev_ws = false;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'#' && prev_ws {
            return value[..i].trim_end();
        }
        prev_ws = *b == b' ' || *b == b'\t';
    }
    value
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

    #[test]
    fn strips_trailing_inline_comment() {
        let got = parse_env_example("APP_ENV=local          # local, staging");
        assert_eq!(got, vec![pair("APP_ENV", "local")]);
    }

    #[test]
    fn preserves_hash_inside_quoted_value() {
        let got = parse_env_example("KEY=\"foo#bar\"");
        assert_eq!(got, vec![pair("KEY", "\"foo#bar\"")]);
    }

    #[test]
    fn preserves_hash_with_no_leading_space() {
        let got = parse_env_example("KEY=#literal");
        assert_eq!(got, vec![pair("KEY", "#literal")]);
    }

    #[test]
    fn strips_multiple_spaces_before_comment() {
        let got = parse_env_example("APP_DEBUG=true      # Set false in production");
        assert_eq!(got, vec![pair("APP_DEBUG", "true")]);
    }
}
