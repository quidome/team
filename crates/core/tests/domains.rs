//! The direction of use between the domains of the core (decision 0027). Each module is in a
//! domain folder under `src/`, and a domain names in its `crate::` paths only itself and the
//! domains it may use. The test reads the source text, test code included.

#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

/// Each domain and the domains it may use (decision 0027). A domain may always use itself.
const RULES: &[(&str, &[&str])] = &[
    (
        "output",
        &["contribution", "team", "game_list", "events", "shared"],
    ),
    ("contribution", &["team", "events", "shared"]),
    ("team", &["events", "shared"]),
    ("game_list", &["events", "shared"]),
    ("events", &["shared"]),
    ("shared", &[]),
];

fn allowed(domain: &str) -> Option<&'static [&'static str]> {
    RULES
        .iter()
        .find(|(name, _)| *name == domain)
        .map(|(_, may_use)| *may_use)
}

/// The first name after each `crate::` (also `$crate::` in a macro) in `source`, with its line
/// (from 1). In a group, `crate::{events::game, output::x}`, the first name of each part.
fn crate_names(source: &str) -> Vec<(usize, &str)> {
    let mut names = Vec::new();
    for (at, _) in source.match_indices("crate::") {
        let before = source[..at].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let start = at + "crate::".len();
        let rest = &source[start..];
        let skipped = rest.len() - rest.trim_start().len();
        let start = start + skipped;
        if source[start..].starts_with('{') {
            group_names(source, start + 1, &mut names);
        } else {
            names.push(name_at(source, start));
        }
    }
    names
}

/// The first name of each part of the group that starts at `start` (just after its `{`).
fn group_names<'a>(source: &'a str, start: usize, names: &mut Vec<(usize, &'a str)>) {
    let mut depth = 0;
    // At the start of a part, before its first name.
    let mut new_part = true;
    for (offset, c) in source[start..].char_indices() {
        let at = start + offset;
        if new_part && depth == 0 && !c.is_whitespace() {
            new_part = false;
            if c != '}' && c != ',' {
                names.push(name_at(source, at));
            }
        }
        match c {
            '{' => depth += 1,
            '}' if depth == 0 => return,
            '}' => depth -= 1,
            ',' if depth == 0 => new_part = true,
            _ => {}
        }
    }
}

/// The name that starts at `at`, with its line.
fn name_at(source: &str, at: usize) -> (usize, &str) {
    let rest = &source[at..];
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    let line = source[..at].matches('\n').count() + 1;
    (line, &rest[..end])
}

/// The breaks of the rule in the file `file` of `domain`, one line each. Three forms can reach
/// another domain past the check of the `crate::` names, so they are breaks too: `super::super`,
/// `super::` in the `mod.rs` of a domain (its parent is the crate root), and another name for
/// the crate (`crate as`).
fn problems_in(domain: &str, file: &str, source: &str) -> Vec<String> {
    let may_use = allowed(domain).unwrap_or(&[]);
    let domain_root = file.ends_with("mod.rs");
    let around = source.lines().zip(1..).filter_map(move |(text, line)| {
        let form = if text.contains("super::super") {
            "super::super"
        } else if domain_root && text.contains("super::") {
            "super:: in mod.rs"
        } else if text.contains("crate as") {
            "crate as"
        } else {
            return None;
        };
        Some(format!(
            "{file}:{line}: `{form}` reaches the crate root past the check; use \
             `crate::<domain>` (decision 0027)"
        ))
    });
    let named = crate_names(source).into_iter().filter_map(|(line, name)| {
        if name == domain || may_use.contains(&name) {
            None
        } else if allowed(name).is_some() {
            Some(format!(
                "{file}:{line}: {domain} uses {name}; {domain} may use {} (decision 0027)",
                may_use_text(may_use)
            ))
        } else {
            Some(format!(
                "{file}:{line}: `crate::{name}` is not a domain; use the path of the domain, \
                     not the flat name (decision 0027)"
            ))
        }
    });
    around.chain(named).collect()
}

fn may_use_text(domains: &[&str]) -> String {
    if domains.is_empty() {
        "no other domain".to_owned()
    } else {
        format!("only {}", domains.join(", "))
    }
}

/// The `.rs` files below `dir`, sorted.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).expect("readable folder") {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn each_domain_uses_only_the_domains_it_may_use() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let mut entries: Vec<PathBuf> = fs::read_dir(&src)
        .expect("readable src")
        .map(|entry| entry.expect("readable entry").path())
        .collect();
    entries.sort();
    let mut problems = Vec::new();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a file name");
        if !path.is_dir() {
            if name != "lib.rs" {
                problems.push(format!(
                    "src/{name}: a module directly under src/; place it in a domain folder \
                     (decision 0027)"
                ));
            }
            continue;
        }
        if allowed(name).is_none() {
            problems.push(format!(
                "src/{name}/: not a domain of decision 0027; use one of {}",
                RULES
                    .iter()
                    .map(|(domain, _)| *domain)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            continue;
        }
        for file in rust_files(&path) {
            let source = fs::read_to_string(&file).expect("readable file");
            let shown = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();
            problems.extend(problems_in(name, &shown, &source));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn a_forbidden_use_line_is_found() {
    let source = "use crate::team::players::Player;\nuse crate::output::messages::week_message;\n";
    assert_eq!(
        problems_in("contribution", "src/contribution/x.rs", source),
        [
            "src/contribution/x.rs:2: contribution uses output; contribution may use only team, \
          events, shared (decision 0027)"
        ]
    );
}

#[test]
fn each_part_of_a_group_is_found() {
    let source = "use crate::{\n    events::game,\n    output::{messages, start_page},\n};\n";
    assert_eq!(crate_names(source), [(2, "events"), (3, "output")]);
    assert_eq!(
        problems_in("contribution", "x.rs", source).len(),
        1,
        "only output is forbidden"
    );
}

#[test]
fn a_full_path_in_code_is_found() {
    let source = "fn f() {\n    let id = crate::game_list::guard::guard(x);\n}\n";
    assert_eq!(
        problems_in("team", "src/team/x.rs", source),
        ["src/team/x.rs:2: team uses game_list; team may use only events, shared (decision 0027)"]
    );
}

#[test]
fn a_flat_name_and_other_crates_are_told_apart() {
    // `crate::Season` goes around the rule through the re-exports of `lib.rs`.
    let problems = problems_in("events", "x.rs", "use crate::Season;\n");
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("not a domain"), "{problems:?}");
    // `other_crate::` and the domain itself are not breaks.
    let source = "use other_crate::output::x;\nuse crate::events::schedule;\n\
                  use crate::shared::season::Season;\n";
    assert!(problems_in("events", "x.rs", source).is_empty());
    // Shared may use no other domain.
    assert_eq!(
        problems_in("shared", "x.rs", "use crate::events::game::Game;\n"),
        ["x.rs:1: shared uses events; shared may use no other domain (decision 0027)"]
    );
}

#[test]
fn a_macro_path_and_a_path_up_two_levels_are_found() {
    // In a macro of this crate, `$crate::output` is the output of this crate.
    let source = "macro_rules! m {\n    () => { $crate::output::y };\n}\n";
    assert_eq!(
        problems_in("events", "x.rs", source),
        ["x.rs:2: events uses output; events may use only shared (decision 0027)"]
    );
    let source = "fn f() {}\nuse super::super::contribution::attendance::Absence;\n";
    assert_eq!(
        problems_in("events", "x.rs", source),
        [
            "x.rs:2: `super::super` reaches the crate root past the check; use `crate::<domain>` \
          (decision 0027)"
        ]
    );
}

#[test]
fn a_path_up_from_the_mod_rs_of_a_domain_and_another_crate_name_are_found() {
    let source = "pub mod training;\nuse super::contribution::attendance::absent_count;\n";
    assert_eq!(
        problems_in("events", "src/events/mod.rs", source),
        [
            "src/events/mod.rs:2: `super:: in mod.rs` reaches the crate root past the check; use \
          `crate::<domain>` (decision 0027)"
        ]
    );
    // In a module of the domain, `super::` is the domain itself.
    assert!(problems_in("events", "src/events/game.rs", "use super::club::Club;\n").is_empty());
    let source = "use crate as root;\nfn f() { root::output::x(); }\n";
    assert_eq!(
        problems_in("shared", "x.rs", source),
        [
            "x.rs:1: `crate as` reaches the crate root past the check; use `crate::<domain>` \
          (decision 0027)"
        ]
    );
}
