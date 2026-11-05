# vcs-lint

A single command line tool that lints branch names and commit message headers.

vcs-lint has no configuration file and never looks around the repository.
Allowed types and scopes are given as options, so the caller (a git hook, a
task runner such as moon, a CI step) decides where they come from. A monorepo
can pass its module list as scopes; a single-module repository simply omits
them. vcs-lint does not depend on git either: everything it checks is passed
as an argument.

## Usage

```
vcs-lint branch <NAME> [OPTIONS]
vcs-lint commit <FILE> [OPTIONS]
```

### List options

`--type`, `--scope` and `--protected` take lists. Values may be given as
repeated flags, as comma-separated values, or both:

```sh
vcs-lint commit .git/COMMIT_EDITMSG --scope root --scope cli
vcs-lint commit .git/COMMIT_EDITMSG --scope root,cli
vcs-lint commit .git/COMMIT_EDITMSG --scope root,$(ls crates | paste -sd, -)
```

Empty values (`a,,b`, a trailing comma) are ignored, and a list whose values
are all empty counts as omitted.

### Common options

- `--type <TYPE>`: allowed types. Defaults to
  `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`,
  `chore`, `revert`.

### `branch <NAME>`

Lints a branch name of the form `<type>/<name>`:

- `<type>` is one of the allowed types.
- `<name>` is lowercase kebab-case starting with a letter:
  `[a-z][a-z0-9]*(-[a-z0-9]+)*`. It cannot contain another `/`.

Options:

- `--protected <BRANCH>`: branches that must not receive direct commits, such
  as `main`. A protected branch fails with advice to open a pull request, and
  its format is not checked. Nothing is protected by default.

### `commit <FILE>`

Lints the header (first line) of the commit message in `FILE`. The file passed
to git's `commit-msg` hook can be given as is. The body is not checked.

Options:

- `--scope <SCOPE>`: allowed scopes.
  - Given: a scope is required and must be one of them. The header has the
    form `<type>(<scope>): <subject>`.
  - Omitted: a scope is not allowed. The header has the form
    `<type>: <subject>`.

The header must also satisfy:

- `:` is followed by exactly one space.
- The subject is not empty.

Breaking change markers (`!` after the type, `BREAKING CHANGE:` footers) are
not supported.

Before finding the header, vcs-lint skips what git removes from the file after
the hook runs:

- Lines starting with `#` (git comments).
- Everything from the `# ------------------------ >8 ------------------------`
  line on (the diff added by `git commit --verbose`).
- Leading blank lines.

Messages whose header starts with one of the following are accepted without
checks:

- `Merge ` (`Merge pull request ...`, `Merge branch ...`).
- `fixup! `, `squash! `, `amend! ` (temporary commits created by
  `git commit --fixup` and friends, which disappear on autosquash).

A message with no header left fails.

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

```
error: direct commits to `main` are not allowed
  create a branch and open a pull request instead
```

## Exit status

- `0`: no violation.
- `1`: a violation was found, or the commit message file could not be read.
- `2`: the command line is invalid (an unknown option, a missing argument).

## License

See the LICENSE file.
