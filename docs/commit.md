# `vcs-lint commit`

Lints the header (first line) of a commit message file.

```
vcs-lint commit [OPTIONS] <FILE>
```

## Examples

```
$ cat COMMIT_EDITMSG
feat(core): add matcher
$ vcs-lint commit COMMIT_EDITMSG
error: invalid commit header `feat(core): add matcher`
  unexpected scope `core`
    scopes are not used; remove `(core)`
$ vcs-lint commit COMMIT_EDITMSG --scope root,core,cli

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

As a Git `commit-msg` hook, which receives the message file as its first
argument. Comments and the `git commit --verbose` diff in it are skipped (see
[Finding the header](#finding-the-header)), so it can be passed as is:

```sh
#!/bin/sh
exec vcs-lint commit "$1" --scope root,$(ls crates | paste -sd, -)
```

## Arguments

- `<FILE>`: the commit message file. It must be UTF-8 text; a file that
  cannot be read fails with status `2` (see [Exit status](cli.md#exit-status)).

## Options

- `--type <TYPE>`: gives the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--scope <SCOPE>`: gives the [allowed scopes](#definitions). A
  [list option](cli.md#list-options).
- `--scope-mode <MODE>`: whether headers have a scope, `auto`, `required`,
  `optional` or `forbidden` (see [Scope mode](#scope-mode)). `auto` when not
  given.
- `-h`, `--help`: print help.

## Rules

### Definitions

- **Allowed types**: the `--type` values, or `feat`, `fix`, `docs`, `style`,
  `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert` when none are
  given. The same set can be given to
  [`vcs-lint branch`](branch.md#definitions), but nothing requires it.
- **Allowed scopes**: the `--scope` values. Empty when none are given.

Values are matched exactly, including case.

### Finding the header

The first line left after skipping the following is the header:

```
                                                        skipped: leading blank line
# Please enter the commit message for your changes.     skipped: starts with `#`
feat: add matcher                                       header
                                                        not read
Body text.                                              not read
# ------------------------ >8 ------------------------   skipped: this line and all after it
diff --git a/src/lib.rs b/src/lib.rs                    skipped
```

A message with no line left fails with `commit message is empty`.

### Messages accepted as is

| Header | Result |
|---|---|
| `Merge branch 'main' into feat/x` | accepted: starts with `Merge ` |
| `fixup! feat: add matcher` | accepted: also `squash! ` and `amend! ` |
| `Merged x` | linted: no exact prefix |
| `fixup!feat: x` | linted: no space after `!` |
| `Revert "feat: add matcher"` | linted: malformed header |

### Scope mode

`--scope-mode` decides whether the header has a scope:

| `--scope-mode` | Header |
|---|---|
| `required` | `<type>(<scope>): <subject>` |
| `optional` | `<type>(<scope>): <subject>` or `<type>: <subject>` |
| `forbidden` | `<type>: <subject>` |
| `auto`, or not given | as `required` when any scope is given, as `forbidden` otherwise |

`required` and `optional` need at least one allowed scope; without one, the
command fails with status `2` (see [Exit status](cli.md#exit-status)).
`forbidden` ignores the `--scope` values.

A malformed header is reported with the form of the mode, written
`<type>[(<scope>)]: <subject>` for `optional`.

### Header

The header takes the form of the [scope mode](#scope-mode), where:

- `<type>` is one of the allowed types.
- `<scope>` is one of the allowed scopes.
- `:` is followed by exactly one space.
- `<subject>` is not empty.

With allowed types `feat`, `fix` and no allowed scopes, so with scopes
forbidden:

| Header | Result |
|---|---|
| `feat: add matcher` | OK |
| `feat(core): add matcher` | unexpected scope `core` |
| `feature: add matcher` | unknown type `feature` |
| `feat:add matcher` | not exactly one space after `:` |
| `feat:  add matcher` | not exactly one space after `:` |
| `feat: ` | empty subject |
| `add matcher` | malformed header |
| `feat!: add matcher` | malformed header: `!` is not recognized |

With allowed types `feat`, `fix` and allowed scopes `root`, `core`, so with
scopes required:

| Header | Result |
|---|---|
| `feat(core): add matcher` | OK |
| `feat: add matcher` | missing scope |
| `feat(parser): add matcher` | unknown scope `parser` |
| `feat(): add matcher` | unknown scope (empty) |
| `feature(parser):add matcher` | unknown type, unknown scope and spacing, reported together |

With allowed types `feat`, `fix`, allowed scopes `root`, `core` and
`--scope-mode optional`:

| Header | Result |
|---|---|
| `feat(core): add matcher` | OK |
| `feat: add matcher` | OK |
| `feat(parser): add matcher` | unknown scope `parser` |
| `feat(): add matcher` | unknown scope (empty) |
