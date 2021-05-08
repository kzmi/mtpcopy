/// File name matching with wildcard
#[derive(Debug, PartialEq, Eq)]
pub struct FileNamePattern {
    pattern: Vec<char>,
}

impl FileNamePattern {
    /// Checks whether a pattern contains any wildcard character.
    pub fn has_wildcard(pattern: &str) -> bool {
        pattern.chars().find(|p| *p == '*' || *p == '?').is_some()
    }

    /// Creates `FileNamePattern`.
    ///
    /// `pattern` can contain wildcard character '*' or '?'.
    pub fn new(pattern: &str) -> FileNamePattern {
        FileNamePattern {
            pattern: pattern.chars().collect(),
        }
    }

    /// Checks whether whole of a text matches this pattern
    pub fn matches(&self, s: &str) -> bool {
        let seq: Vec<char> = s.chars().collect();
        matches_seq(&seq, &self.pattern)
    }

    /// Returns pattern string.
    #[allow(dead_code)]
    pub fn get_pattern(&self) -> String {
        self.pattern.iter().collect()
    }
}

fn matches_seq(mut seq: &[char], mut pattern: &[char]) -> bool {
    while pattern.len() > 0 {
        let pat = pattern[0];
        pattern = &pattern[1..];
        match pat {
            '*' => {
                if pattern.len() == 0 {
                    // last '*' matches any remaining sequence
                    return true;
                }
                loop {
                    if matches_seq(seq, pattern) {
                        return true;
                    }
                    if seq.len() == 0 {
                        break;
                    }
                    seq = &seq[1..];
                }
                return false;
            }
            '?' => {
                if seq.len() == 0 {
                    return false;
                }
                seq = &seq[1..];
            }
            _ => {
                if seq.len() == 0 || seq[0] != pat {
                    return false;
                }
                seq = &seq[1..];
            }
        }
    }
    seq.len() == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_matches_seq(seq: &str, pattern: &str) -> bool {
        let seq_v: Vec<char> = seq.chars().collect();
        let pat_v: Vec<char> = pattern.chars().collect();
        matches_seq(seq_v.as_slice(), pat_v.as_slice())
    }

    #[test]
    fn test_matches_seq() {
        assert_eq!(true, call_matches_seq("", ""));
        assert_eq!(true, call_matches_seq("", "*"));
        assert_eq!(true, call_matches_seq("", "******"));
        assert_eq!(false, call_matches_seq("", "?"));
        assert_eq!(false, call_matches_seq("", "a"));

        assert_eq!(false, call_matches_seq("a", ""));
        assert_eq!(true, call_matches_seq("a", "*"));
        assert_eq!(true, call_matches_seq("a", "******"));
        assert_eq!(true, call_matches_seq("a", "?"));
        assert_eq!(false, call_matches_seq("a", "??"));
        assert_eq!(true, call_matches_seq("a", "a"));
        assert_eq!(false, call_matches_seq("a", "aa"));

        assert_eq!(true, call_matches_seq("abc", "a*"));
        assert_eq!(true, call_matches_seq("abc", "a*c"));
        assert_eq!(true, call_matches_seq("abc", "a******c"));
        assert_eq!(false, call_matches_seq("abc", "a*x"));
        assert_eq!(true, call_matches_seq("abc", "a*b*"));
        assert_eq!(true, call_matches_seq("abc", "a*b*c"));
        assert_eq!(false, call_matches_seq("abc", "a*b*cx"));
        assert_eq!(true, call_matches_seq("abc", "?bc"));
        assert_eq!(true, call_matches_seq("abc", "a?c"));
        assert_eq!(true, call_matches_seq("abc", "ab?"));
        assert_eq!(false, call_matches_seq("abc", "ab?x"));

        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "ab?a*c"));
        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "ab?*abc*abc"));
        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "ab?*********abc"));
        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "ab?*******??abc"));
        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "ab?***a?***?abc"));
        assert_eq!(true, call_matches_seq("abcabcabcabcabc", "*a*a*a*a*a*c"));
        assert_eq!(false, call_matches_seq("abcabcabcabcabc", "*a*a*a*a*a*a*c"));
    }

    #[test]
    fn test_file_name_pattern() {
        let pat = FileNamePattern::new("a?c*c");
        assert_eq!(false, pat.matches(""));
        assert_eq!(false, pat.matches("x"));
        assert_eq!(true, pat.matches("abcc"));
        assert_eq!(true, pat.matches("acccc"));
    }
}
