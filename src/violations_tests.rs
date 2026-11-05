use super::Violations;

#[test]
fn collect_succeeds_without_failures() {
    let results: [Result<(), &str>; 2] = [Ok(()), Ok(())];
    assert_eq!(Violations::collect(results), Ok(()));
}

#[test]
fn collect_succeeds_without_results() {
    assert_eq!(Violations::<&str>::collect([]), Ok(()));
}

#[test]
fn collect_keeps_every_failure_in_order() {
    let results = [Err("a"), Ok(()), Err("b")];
    assert_eq!(
        Violations::collect(results),
        Err(Violations(vec!["a", "b"]))
    );
}

#[test]
fn indents_every_line_of_every_violation() {
    let violations = Violations(vec!["first\n  hint", "second"]);
    assert_eq!(violations.to_string(), "  first\n    hint\n  second");
}
