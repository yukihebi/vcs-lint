use super::{HintSyntaxError, Hinted};
use crate::hint::Hint;

fn hint(text: &str) -> Hint {
    Hint::parse(text).unwrap()
}

#[test]
fn prefixes_each_line_and_leaves_blank_lines_bare() {
    assert_eq!(
        hint("rename it:\n\n  git branch -m NEW\n").to_string(),
        "hint: rename it:\nhint:\nhint:   git branch -m NEW"
    );
}

#[test]
fn follows_report_with_each_hint() {
    let hinted = Hinted {
        report: "invalid branch name `x`\n  missing type prefix",
        hints: vec![hint("types"), hint("names")],
    };
    assert_eq!(
        hinted.to_string(),
        "invalid branch name `x`\n  missing type prefix\nhint: types\nhint: names"
    );
}

#[test]
fn leaves_report_alone_without_hints() {
    let hinted = Hinted {
        report: "branch `main` is protected",
        hints: Vec::new(),
    };
    assert_eq!(hinted.to_string(), "branch `main` is protected");
}

#[test]
fn words_placeholder_with_its_braces() {
    let err = HintSyntaxError::Placeholder {
        name: "branch".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "placeholder `{branch}` is not supported; write `{{` and `}}` for braces"
    );
}
