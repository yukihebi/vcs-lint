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
exec vcs-lint commit "$1" --scope root --scope-dir crates
```

## Arguments

- `<FILE>`: the commit message file. It must be UTF-8 text; a file that
  cannot be read fails with status `2` (see [Exit status](cli.md#exit-status)).

## Options

- `--type <TYPE>`: adds to the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--type-dir <DIR>`: adds the directories under `DIR` to the
  [allowed types](#definitions) (see [Directories](cli.md#directories)). A
  [list option](cli.md#list-options).
- `--type-exclude <TYPE>`: excludes from the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--type-defaults <MODE>`: when the default types are included in the
  [allowed types](#definitions), `auto`, `always` or `never` (see
  [Defaults](cli.md#defaults)). `auto` when not given.
- `--scope <SCOPE>`: adds to the [allowed scopes](#definitions). A
  [list option](cli.md#list-options).
- `--scope-dir <DIR>`: adds the directories under `DIR` to the
  [allowed scopes](#definitions) (see [Directories](cli.md#directories)). A
  [list option](cli.md#list-options).
- `--scope-exclude <SCOPE>`: excludes from the [allowed scopes](#definitions).
  A [list option](cli.md#list-options).
- `--scope-mode <MODE>`: whether headers have a scope, `auto`, `required`,
  `optional` or `forbidden` (see [Scope mode](#scope-mode)). `auto` when not
  given.
- `--config <PATH>`: reads options from a
  [config file](cli.md#config-file). The file named by
  [`VCS_LINT_CONFIG`](cli.md#vcs_lint_config) when not given.
- `-h`, `--help`: print help.

## Rules

### Definitions

- **Allowed types**: [composed](cli.md#composition) from the default types
  `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`,
  `chore`, `revert`, the `--type` values, the `--type-dir` directories and the
  `--type-exclude` values. By default, the default types are included only when
  neither `--type` nor `--type-dir` is given (see
  [Defaults](cli.md#defaults)). When no type is left, nothing could pass, so the
  command fails with status `2`. The same set can be given to
  [`vcs-lint branch`](branch.md#definitions), but nothing requires it.
- **Allowed scopes**: [composed](cli.md#composition) from the `--scope` values,
  the `--scope-dir` directories and the `--scope-exclude` values. Empty when
  none are left.

Values are matched exactly, including case.

Values no header can match fail with status `2` (see
[Values nothing can match](cli.md#values-nothing-can-match)):

- An allowed type cannot contain `(`, `)`, `:`, `!` or whitespace, nor start
  with `#`, which starts a comment line.
- An allowed scope cannot contain a line break, and its parentheses must be
  balanced, as in `core(unix)`.

Allowed scopes are not checked when scopes are forbidden, as they are ignored.

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

Scopes count as given once `--scope` or `--scope-dir` appears, even when every
given scope is excluded. `required` and `optional`, and `auto` with scopes
given, need at least one allowed scope; without one, the command fails with
status `2` (see [Exit status](cli.md#exit-status)).
`forbidden` ignores the `--scope` values and the `--scope-dir` directories.

A malformed header is reported with the form of the mode, written
`<type>[(<scope>)]: <subject>` for `optional`.

### Header

The header takes the form of the [scope mode](#scope-mode), where:

- `<type>` is one of the allowed types.
- `<scope>` is one of the allowed scopes. It ends at the `)` closing its `(`,
  so it may hold balanced parentheses.
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
| `feat(core(unix)): add matcher` | unknown scope `core(unix)` |
| `feat(core: add matcher` | malformed header: `(` is not closed |
| `feature(parser):add matcher` | unknown type, unknown scope and spacing, reported together |

With allowed types `feat`, `fix`, allowed scopes `root`, `core` and
`--scope-mode optional`:

| Header | Result |
|---|---|
| `feat(core): add matcher` | OK |
| `feat: add matcher` | OK |
| `feat(parser): add matcher` | unknown scope `parser` |
| `feat(): add matcher` | unknown scope (empty) |
