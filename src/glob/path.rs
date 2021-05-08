use std::fmt::Debug;

use super::filename::FileNamePattern;

#[derive(Debug, Eq, PartialEq)]
pub enum PathMatchingState {
    /// the path component was rejected
    Rejected,
    /// the path component was accepted
    Accepted,
    /// the path matching was completed
    Completed,
}

/// Creates linked matchers that match the given path pattern.
///
/// * `pattern` - path pattern.  
///     Each component can contain wildcard characters ('*' and '?').  
///     `**` matches zero or more any directories.
pub fn create_path_pattern_matcher(
    pattern: &str,
) -> Result<RootPathMatcher, Box<dyn std::error::Error>> {
    if pattern.len() == 0 {
        return Err("path is empty.".into());
    }

    let pat_str = pattern.to_string();
    let components: Vec<&str> = if pat_str == "" {
        Vec::<&str>::new()
    } else {
        pat_str.split(&['/', '\\'][..]).collect()
    };

    let mut next: Option<Box<PathMatcher>> = None;
    let mut must_be_dir = false;
    for compo in components.into_iter().rev() {
        if compo == "" {
            continue;
        }
        if compo == "." || compo == ".." {
            return Err(format!("\"{}\" in the path is not allowed.", compo).into());
        }
        if compo == "**" {
            if next.is_none() {
                return Err("the path ending with \"**\" is not allowed.".into());
            }
            next = Some(Box::new(PathMatcher::AnyDirectoriesMatcher {
                next: next.unwrap(),
            }));
        } else {
            if FileNamePattern::has_wildcard(compo) {
                next = Some(Box::new(PathMatcher::FileNamePatternMatcher {
                    pattern: FileNamePattern::new(compo),
                    must_be_dir,
                    next,
                }));
            } else {
                next = Some(Box::new(PathMatcher::ExactNameMatcher {
                    name: compo.to_string(),
                    must_be_dir,
                    next,
                }));
            }
        }
        must_be_dir = true;
    }

    Ok(RootPathMatcher { next })
}

/// A matcher that matches the root
pub struct RootPathMatcher {
    next: Option<Box<PathMatcher>>,
}

impl RootPathMatcher {
    pub fn matches_root(&self) -> (PathMatchingState, Option<&PathMatcher>) {
        match &self.next {
            None => (PathMatchingState::Completed, None),
            Some(m) => (PathMatchingState::Accepted, Some(&m)),
        }
    }

    // for inspection in testing
    #[allow(dead_code)]
    fn next_matcher(&self) -> Option<&PathMatcher> {
        match &self.next {
            None => None,
            Some(m) => Some(&m),
        }
    }
}

/// Other matchers
#[derive(Debug)]
pub enum PathMatcher {
    ExactNameMatcher {
        name: String,
        must_be_dir: bool,
        next: Option<Box<PathMatcher>>,
    },
    FileNamePatternMatcher {
        pattern: FileNamePattern,
        must_be_dir: bool,
        next: Option<Box<PathMatcher>>,
    },
    AnyDirectoriesMatcher {
        next: Box<PathMatcher>,
    },
}

impl PathMatcher {
    pub fn matches(&self, name: &str, is_dir: bool) -> (PathMatchingState, Option<&PathMatcher>) {
        match &self {
            PathMatcher::ExactNameMatcher {
                name: m_name,
                must_be_dir,
                next,
            } => {
                if (!*must_be_dir || is_dir) && name == m_name {
                    match next {
                        None => (PathMatchingState::Completed, None),
                        Some(m) => (PathMatchingState::Accepted, Some(&m)),
                    }
                } else {
                    (PathMatchingState::Rejected, None)
                }
            }

            PathMatcher::FileNamePatternMatcher {
                pattern,
                must_be_dir,
                next,
            } => {
                if (!*must_be_dir || is_dir) && pattern.matches(name) {
                    match next {
                        None => (PathMatchingState::Completed, None),
                        Some(m) => (PathMatchingState::Accepted, Some(&m)),
                    }
                } else {
                    (PathMatchingState::Rejected, None)
                }
            }

            PathMatcher::AnyDirectoriesMatcher { next } => {
                let (next_state, next_matcher) = next.matches(name, is_dir);
                match next_state {
                    PathMatchingState::Rejected => {
                        if is_dir {
                            (PathMatchingState::Accepted, Some(self))
                        } else {
                            (PathMatchingState::Rejected, None)
                        }
                    }
                    _ => (next_state, next_matcher),
                }
            }
        }
    }

    // for inspection in testing
    #[allow(dead_code)]
    fn next_matcher(&self) -> Option<&PathMatcher> {
        match &self {
            PathMatcher::ExactNameMatcher {
                name: _,
                must_be_dir: _,
                next,
            } => match next {
                None => None,
                Some(m) => Some(&m),
            },

            PathMatcher::FileNamePatternMatcher {
                pattern: _,
                must_be_dir: _,
                next,
            } => match next {
                None => None,
                Some(m) => Some(&m),
            },

            PathMatcher::AnyDirectoriesMatcher { next } => Some(next),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn assert_path_matchers(
        result: &Result<RootPathMatcher, Box<dyn std::error::Error>>,
        expected: &[PathMatcher],
    ) {
        let root_matcher = result.as_ref().unwrap();
        let mut next_matcher: Option<&PathMatcher> = root_matcher.next_matcher();
        let mut expected_iter = expected.iter();
        while let Some(matcher) = next_matcher {
            let expected_matcher = expected_iter.next().unwrap();
            match matcher {
                PathMatcher::ExactNameMatcher {
                    name: actual_name,
                    must_be_dir: actual_must_be_dir,
                    next: _,
                } => match expected_matcher {
                    PathMatcher::ExactNameMatcher {
                        name: expected_name,
                        must_be_dir: expected_must_be_dir,
                        next: _,
                    } => {
                        assert_eq!(*expected_must_be_dir, *actual_must_be_dir);
                        assert_eq!(expected_name, actual_name);
                    }
                    _ => panic!(),
                },

                PathMatcher::FileNamePatternMatcher {
                    pattern: actual_pattern,
                    must_be_dir: actual_must_be_dir,
                    next: _,
                } => match expected_matcher {
                    PathMatcher::FileNamePatternMatcher {
                        pattern: expected_pattern,
                        must_be_dir: expected_must_be_dir,
                        next: _,
                    } => {
                        assert_eq!(*expected_must_be_dir, *actual_must_be_dir);
                        assert_eq!(expected_pattern, actual_pattern);
                    }
                    _ => panic!(),
                },

                PathMatcher::AnyDirectoriesMatcher { next: _ } => match expected_matcher {
                    PathMatcher::AnyDirectoriesMatcher { next: _ } => (),
                    _ => panic!(),
                },
                // _ => panic!(),
            }
            next_matcher = matcher.next_matcher();
        }
        assert!(expected_iter.next().is_none());
    }

    #[test_case("/" ; "root")]
    #[test_case("///" ; "duplicated separators")]
    fn test_create_path_pattern_matcher_root_dir(pattern: &str) {
        let expected = [];
        let result = create_path_pattern_matcher(pattern);
        assert_path_matchers(&result, &expected[..]);
    }

    #[test_case("aaa" ; "relative path")]
    #[test_case("/aaa" ; "absolute path")]
    #[test_case("///aaa" ; "absolute path with duplicated separators")]
    #[test_case("aaa/" ; "relative path ending with separator")]
    #[test_case("aaa///" ; "relative path ending with duplicated separator")]
    #[test_case("aaa" ; "relative path to a file")]
    fn test_create_path_pattern_matcher_exact_name(pattern: &str) {
        let expected = [PathMatcher::ExactNameMatcher {
            name: "aaa".to_string(),
            must_be_dir: false,
            next: None,
        }];
        let result = create_path_pattern_matcher(pattern);
        assert_path_matchers(&result, &expected[..]);
    }

    #[test_case("a*a", "a*a" ; "path with wildcard 1")]
    #[test_case("aa?aa", "aa?aa" ; "path with wildcard 2")]
    #[test_case("a?a*a", "a?a*a" ; "path with wildcard 3")]
    #[test_case("/a*a", "a*a" ; "absolute path with wildcard")]
    #[test_case("a*a", "a*a" ; "path to a file with wildcard")]
    fn test_create_path_pattern_matcher_wildcard(pattern: &str, expected_filename_pattern: &str) {
        let expected = [PathMatcher::FileNamePatternMatcher {
            pattern: FileNamePattern::new(expected_filename_pattern),
            must_be_dir: false,
            next: None,
        }];
        let result = create_path_pattern_matcher(pattern);
        assert_path_matchers(&result, &expected[..]);
    }

    #[test_case("**/aaa", "aaa" ; "path with wildcard 2")]
    fn test_create_path_pattern_matcher_any_dir(pattern: &str, expected_filename_pattern: &str) {
        let expected = [
            PathMatcher::AnyDirectoriesMatcher {
                next: // this value is not referenced in `assert_path_matchers()`
                    Box::new(PathMatcher::ExactNameMatcher {
                        name: expected_filename_pattern.to_string(),
                        must_be_dir: false,
                        next: None,
                    }),
            },
            PathMatcher::ExactNameMatcher {
                name: expected_filename_pattern.to_string(),
                must_be_dir: false,
                next: None,
            },
        ];
        let result = create_path_pattern_matcher(pattern);
        assert_path_matchers(&result, &expected[..]);
    }

    #[test]
    fn test_create_path_pattern_matcher_multiple_components() {
        let expected = [
            PathMatcher::ExactNameMatcher {
                name: "aaa".to_string(),
                must_be_dir: true,
                next: None,
            },
            PathMatcher::FileNamePatternMatcher {
                pattern: FileNamePattern::new("b*b"),
                must_be_dir: true,
                next: None,
            },
            PathMatcher::AnyDirectoriesMatcher {
                next: // this value is not referenced in `assert_path_matchers()`
                    Box::new(PathMatcher::ExactNameMatcher {
                        name: "ccc".to_string(),
                        must_be_dir: false,
                        next: None,
                    }),
            },
            PathMatcher::ExactNameMatcher {
                name: "ccc".to_string(),
                must_be_dir: false,
                next: None,
            },
        ];
        let result = create_path_pattern_matcher("aaa/b*b/**/ccc");
        assert_path_matchers(&result, &expected[..]);
    }

    #[test]
    fn test_create_path_pattern_matcher_errors() {
        assert!(matches!(create_path_pattern_matcher(""), Err(_)));
        assert!(matches!(create_path_pattern_matcher("a/./a"), Err(_)));
        assert!(matches!(create_path_pattern_matcher("a/../a"), Err(_)));
        assert!(matches!(create_path_pattern_matcher("a/**"), Err(_)));
    }

    #[test]
    fn test_path_pattern_scenario_root() {
        let root_matcher = create_path_pattern_matcher("/").unwrap();
        let current_matcher = &root_matcher;

        let (state, next_matcher) = current_matcher.matches_root();
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());
    }

    #[test]
    fn test_path_pattern_scenario_exect_name() {
        let root_matcher = create_path_pattern_matcher("/aaa/bbb").unwrap();
        // root
        let (state, next_matcher) = root_matcher.matches_root();
        assert_eq!(PathMatchingState::Accepted, state);
        let mut current_matcher = next_matcher.unwrap();

        // 1st component

        // file "xxx" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("xxx", false);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // directory "xxx" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("xxx", true);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // file "aaa" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("aaa", false);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // directory "aaa" matches the pattern
        let (state, next_matcher) = current_matcher.matches("aaa", true);
        assert_eq!(PathMatchingState::Accepted, state);
        current_matcher = next_matcher.unwrap();

        // 2nd component

        // file "bbb" matches the pattern
        let (state, next_matcher) = current_matcher.matches("bbb", false);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // also directory "bbb" matches the pattern
        let (state, next_matcher) = current_matcher.matches("bbb", true);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());
    }

    #[test]
    fn test_path_pattern_scenario_file_name_pattern() {
        let root_matcher = create_path_pattern_matcher("/a*a/b*b").unwrap();
        // root
        let (state, next_matcher) = root_matcher.matches_root();
        assert_eq!(PathMatchingState::Accepted, state);
        let mut current_matcher = next_matcher.unwrap();

        // 1st component

        // file "xxx" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("xxx", false);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // directory "xxx" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("xxx", true);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // file "aaaaaa" doesn't match the pattern
        let (state, next_matcher) = current_matcher.matches("aaaaaa", false);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // directory "aaaaaa" matches the pattern
        let (state, next_matcher) = current_matcher.matches("aaaaaa", true);
        assert_eq!(PathMatchingState::Accepted, state);
        current_matcher = next_matcher.unwrap();

        // 2nd component

        // file "bbbbbb" matches the pattern
        let (state, next_matcher) = current_matcher.matches("bbbbbb", false);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // also directory "bbbbbb" matches the pattern
        let (state, next_matcher) = current_matcher.matches("bbbbbb", true);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());
    }

    #[test]
    fn test_path_pattern_scenario_any_directory() {
        let root_matcher = create_path_pattern_matcher("/aaa/**/bbb").unwrap();

        // root
        let (state, next_matcher) = root_matcher.matches_root();
        assert_eq!(PathMatchingState::Accepted, state);
        let mut base_matcher = next_matcher.unwrap();

        // 1st component

        // directory "aaa" matches the pattern
        let (state, next_matcher) = base_matcher.matches("aaa", true);
        assert_eq!(PathMatchingState::Accepted, state);
        base_matcher = next_matcher.unwrap();
        // 2nd component ("**")

        // file "bbb" matches the pattern (/aaa/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", false);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // also directory "bbb" matches the pattern (/aaa/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", true);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());
        // file "ccc" doesn't match the pattern
        let (state, next_matcher) = base_matcher.matches("ccc", false);
        assert_eq!(PathMatchingState::Rejected, state);
        assert!(next_matcher.is_none());

        // directory "ccc" matches the pattern
        let (state, next_matcher) = base_matcher.matches("ccc", true);
        assert_eq!(PathMatchingState::Accepted, state);
        base_matcher = next_matcher.unwrap();
        // 3rd component ("**")

        // file "bbb" matches the pattern (/aaa/ccc/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", false);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // also directory "bbb" matches the pattern (/aaa/ccc/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", true);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // directory "ddd" matches the pattern
        let (state, next_matcher) = base_matcher.matches("ddd", true);
        assert_eq!(PathMatchingState::Accepted, state);
        base_matcher = next_matcher.unwrap();

        // file "bbb" matches the pattern (/aaa/ccc/ddd/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", false);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());

        // also directory "bbb" matches the pattern (/aaa/ccc/ddd/bbb)
        let (state, next_matcher) = base_matcher.matches("bbb", true);
        assert_eq!(PathMatchingState::Completed, state);
        assert!(next_matcher.is_none());
    }
}
