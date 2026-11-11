use super::{Hint, Hints};
use crate::report::hint::HintSyntaxError;

/// Rules of a command, standing for those of any command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rule {
    Type,
    Scope,
    Format,
    Protected,
}

fn hint(lines: &[&str]) -> Hint {
    Hint(lines.iter().map(|&line| line.to_owned()).collect())
}

fn placeholder(name: &str) -> Result<Hint, HintSyntaxError> {
    Err(HintSyntaxError::Placeholder {
        name: name.to_owned(),
    })
}

#[test]
fn keeps_text_without_braces() {
    assert_eq!(
        Hint::parse("see CONTRIBUTING.md"),
        Ok(hint(&["see CONTRIBUTING.md"]))
    );
}

#[test]
fn unescapes_doubled_braces() {
    assert_eq!(Hint::parse("{{x}} }}{{"), Ok(hint(&["{x} }{"])));
}

#[test]
fn rejects_empty_text() {
    assert_eq!(Hint::parse(""), Err(HintSyntaxError::Empty));
}

#[test]
fn rejects_every_placeholder_by_its_name() {
    assert_eq!(Hint::parse("git branch -m {branch}"), placeholder("branch"));
    assert_eq!(Hint::parse("{branch:>8}"), placeholder("branch"));
    assert_eq!(Hint::parse("{{ok}} {0}"), placeholder("0"));
}

#[test]
fn rejects_brace_opening_or_closing_no_placeholder() {
    for text in ["a { b", "a } b", "{}", "{{x}"] {
        assert_eq!(Hint::parse(text), Err(HintSyntaxError::Brace), "{text:?}");
    }
}

#[test]
fn splits_lines_without_one_after_final_line_break() {
    assert_eq!(Hint::parse("a\r\n\nb\n"), Ok(hint(&["a", "", "b"])));
    assert_eq!(
        Hint::parse("see CONTRIBUTING.md\n"),
        Hint::parse("see CONTRIBUTING.md")
    );
    assert_eq!(Hint::parse("\n"), Ok(hint(&[""])));
}

#[test]
fn gives_hints_of_broken_rules_in_their_order() {
    let hints = Hints::new([
        (Rule::Type, Some(hint(&["types"]))),
        (Rule::Scope, None),
        (Rule::Format, Some(hint(&["format"]))),
    ]);
    assert_eq!(
        hints.for_rules(&[Rule::Format, Rule::Scope, Rule::Type]),
        [hint(&["format"]), hint(&["types"])]
    );
    assert_eq!(hints.for_rules(&[Rule::Protected]), []);
}

#[test]
fn gives_hint_shared_by_broken_rules_once() {
    let hints = Hints::new([
        (Rule::Type, Some(hint(&["see CONTRIBUTING.md"]))),
        (Rule::Scope, Some(hint(&["scopes"]))),
        (Rule::Format, Some(hint(&["see CONTRIBUTING.md"]))),
    ]);
    assert_eq!(
        hints.for_rules(&[Rule::Type, Rule::Scope, Rule::Format]),
        [hint(&["see CONTRIBUTING.md"]), hint(&["scopes"])]
    );
}
