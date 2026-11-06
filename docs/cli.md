# Command line

```
vcs-lint branch <NAME> [OPTIONS]
vcs-lint commit <FILE> [OPTIONS]
```

The rules of each subcommand are described in [branch.md](branch.md) and
[commit.md](commit.md).

## List options

`--type`, `--scope` and `--protected` take lists. Values may be given as
repeated flags, as comma-separated values, or both:

```sh
vcs-lint commit .git/COMMIT_EDITMSG --scope root --scope cli
vcs-lint commit .git/COMMIT_EDITMSG --scope root,cli
vcs-lint commit .git/COMMIT_EDITMSG --scope root,$(ls crates | paste -sd, -)
```

- Whitespace around each value is trimmed.
- Empty values (`a,,b`, a trailing comma) are ignored.
- A list whose values are all empty counts as omitted.

## Common options

- `--type <TYPE>`: allowed types. Defaults to `feat`, `fix`, `docs`, `style`,
  `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.

## Output

Violations are written to standard error. Each one names the offending input,
what is wrong with it, and the values that would be accepted. All violations
of an input are reported together:

```
error: invalid commit header `feature(parser):add matcher`
  unknown type `feature`
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
  unknown scope `parser`
    allowed scopes: root, core
  expected exactly one space after `:`
```

## Exit status

- `0`: no violation.
- `1`: a violation was found, or the commit message file could not be read.
- `2`: the command line is invalid, such as an unknown option or a missing
  argument.
