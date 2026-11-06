# `vcs-lint commit`

Lints the header (first line) of a commit message file.

```
vcs-lint commit [OPTIONS] <FILE>
```

## Examples

As a Git `commit-msg` hook, which receives the message file as its first
argument:

```sh
#!/bin/sh
exec vcs-lint commit "$1" --scope root,$(ls crates | paste -sd, -)
```

Without `--scope`, a header must not have a scope:

```
$ cat COMMIT_EDITMSG
feat(core): add matcher
$ vcs-lint commit COMMIT_EDITMSG
error: invalid commit header `feat(core): add matcher`
  unexpected scope `core`
    scopes are not used; remove `(core)`
$ vcs-lint commit COMMIT_EDITMSG --scope root,core,cli
```

With `--scope`, a header must have one of them:

```
$ cat COMMIT_EDITMSG
feature(parser):add matcher
$ vcs-lint commit COMMIT_EDITMSG --scope root,core,cli
error: invalid commit header `feature(parser):add matcher`
  unknown type `feature`
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
  unknown scope `parser`
    allowed scopes: root, core, cli
  expected exactly one space after `:`
```

Failing runs exit with status `1`.

## Arguments

- `<FILE>`: the commit message file, such as the one given to the
  `commit-msg` hook. It must be UTF-8 text. A file that cannot be read fails
  with status `1`.

## Options

- `--type <TYPE>`: allowed types. A [list option](cli.md#list-options).
  Defaults to `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
  `build`, `ci`, `chore`, `revert`. Pass the same values as to
  [`vcs-lint branch --type`](branch.md#options) to keep branches and commits
  consistent; nothing requires them to match.
- `--scope <SCOPE>`: allowed scopes. A [list option](cli.md#list-options).
  Whether it is given switches the header format; see [Header](#header).
- `-h`, `--help`: print help.

Values are matched exactly, including case.

## Rules

### Finding the header

The `commit-msg` hook receives the file before Git cleans it up, so the
following lines are skipped:

- Lines starting with `#` (Git comments).
- Everything from the `# ------------------------ >8 ------------------------`
  line on (the diff added by `git commit --verbose`).
- Leading blank lines.

The first line left is the header. A message with no line left fails.

### Messages accepted as is

A header starting with one of the following passes without further checks:

- `Merge ` (`Merge pull request ...`, `Merge branch ...`).
- `fixup! `, `squash! `, `amend! ` (temporary commits created by
  `git commit --fixup` and similar, which disappear on autosquash).

### Header

Without `--scope`, the header has the form `<type>: <subject>`, and a scope is
not allowed.

With `--scope`, the header has the form `<type>(<scope>): <subject>`, and the
scope is required:

- `<scope>` is one of the `--scope` values.

In both forms:

- `<type>` is one of the `--type` values.
- `:` is followed by exactly one space.
- `<subject>` is not empty.

An unknown type, a scope problem and a subject problem are each reported, all
at once. A header that does not fit the form at all is reported as malformed,
with the expected form.

### Current limitations

- Only the header is read; the body and footers are not checked.
- Breaking change markers are not recognized. A `!` after the type or scope
  makes the header malformed.
- Other messages Git generates, such as the default `Revert "..."`, are linted
  like any other.
