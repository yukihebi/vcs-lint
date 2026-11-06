# `vcs-lint commit`

```
vcs-lint commit <FILE> [--type <TYPE>]... [--scope <SCOPE>]...
```

Lints the header (first line) of the commit message in `FILE`. The file passed
to git's `commit-msg` hook can be given as is. List options are written as
described in [cli.md](cli.md#list-options).

## Header format

Whether the header has a scope depends on `--scope`:

- Given: a scope is required and must be one of the allowed scopes. The header
  has the form `<type>(<scope>): <subject>`.
- Omitted: a scope is not allowed. The header has the form
  `<type>: <subject>`.

In both forms:

- `<type>` is one of the allowed types (`--type`).
- `:` is followed by exactly one space.
- `<subject>` is not empty.

Every violation in the header is reported together.

## Finding the header

The commit-msg hook receives the file before git cleans it up. vcs-lint skips
what git removes afterwards:

- Lines starting with `#` (git comments).
- Everything from the `# ------------------------ >8 ------------------------`
  line on (the diff added by `git commit --verbose`).
- Leading blank lines.

A message with no header left fails.

## Messages accepted without checks

Messages whose header starts with one of the following pass as is:

- `Merge ` (`Merge pull request ...`, `Merge branch ...`).
- `fixup! `, `squash! `, `amend! ` (temporary commits created by
  `git commit --fixup` and friends, which disappear on autosquash).

## Current limitations

- Only the header is read; the body and footers are not checked.
- Breaking change markers are not recognized. A `!` after the type or scope
  makes the header malformed.
- Other messages git generates, such as the default `Revert "..."`, are linted
  like any other.
