# `vcs-lint branch`

Lints a branch name, or the current branch when no name is given.

```
vcs-lint branch [OPTIONS] [NAME]
```

## Examples

```
$ vcs-lint branch feat/add-matcher --protected main

$ vcs-lint branch main --protected main,develop
error: direct commits to `main` are not allowed
  create a branch and open a pull request instead

$ vcs-lint branch feature/Add_Matcher --type feat,fix
error: invalid branch name `feature/Add_Matcher`
  unknown type `feature`
    allowed types: feat, fix
  invalid name `Add_Matcher`
    expected: lowercase kebab-case starting with a letter, e.g. `add-matcher`

$ vcs-lint branch HEAD --detached deny
error: commits on a detached HEAD are not allowed
  switch to a branch or create one before committing
```

As a Git `pre-commit` hook, linting the [current branch](#current-branch):

```sh
#!/bin/sh
exec vcs-lint branch --protected main --detached deny
```

## Arguments

- `[NAME]`: the branch name to lint, or `HEAD` for a
  [detached HEAD](#detached-head). The [current branch](#current-branch) when
  not given.

## Options

- `--type <TYPE>`: adds to the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--type-exclude <TYPE>`: excludes from the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--type-defaults <MODE>`: when the default types are included in the
  [allowed types](#definitions), `auto`, `always` or `never` (see
  [Defaults](cli.md#defaults)). `auto` when not given.
- `--protected <BRANCH>`: adds to the [protected branches](#definitions). A
  [list option](cli.md#list-options).
- `--protected-exclude <BRANCH>`: excludes from the
  [protected branches](#definitions). A [list option](cli.md#list-options).
- `--detached <RULE>`: whether a [detached HEAD](#detached-head) passes,
  `allow` or `deny`. `allow` when not given.
- `-h`, `--help`: print help.

## Current branch

Without `NAME`, the current branch is read from Git, as `HEAD` of the
repository the current directory belongs to. Git is run as a child process, so
`GIT_DIR` and worktrees are followed as Git itself does.

- A branch, including one with no commit yet, is linted by its name.
- A detached HEAD is linted as a [detached HEAD](#detached-head).
- When Git is not found or the current directory is not in a repository, the
  command fails with status `2`.

The rules are checked first, so rules that cannot be resolved fail with status
`2` without running Git.

```
$ vcs-lint branch
error: `git symbolic-ref -q HEAD` failed with exit status: 128
  fatal: not a git repository (or any of the parent directories): .git
```

## Rules

### Definitions

- **Allowed types**: [composed](cli.md#composition) from the default types
  `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`,
  `chore`, `revert`, the `--type` values and the `--type-exclude` values. By
  default, the default types are included only when no `--type` is given (see
  [Defaults](cli.md#defaults)). When no type is left, nothing could pass, so the
  command fails with status `2`. The same set can be given to
  [`vcs-lint commit`](commit.md#definitions), but nothing requires it.
- **Protected branches**: [composed](cli.md#composition) from the
  `--protected` values and the `--protected-exclude` values. Empty when none
  are left.

Values are matched exactly, including case.

Values no branch name can match fail with status `2` (see
[Values nothing can match](cli.md#values-nothing-can-match)):

- An allowed type cannot contain `/`, as a branch name is split at its first
  `/`, nor a line break.
- A protected branch cannot contain a line break.

### Detached HEAD

A `NAME` of exactly `HEAD` stands for a detached HEAD, as Git refuses `HEAD`
as a branch name, and so does a detached HEAD read as the
[current branch](#current-branch). A detached HEAD has no branch name, so neither the protected
branches nor the format apply to it; `--detached` alone decides whether it
passes.

| `--detached` | Result for `HEAD` |
|---|---|
| `allow`, or not given | OK |
| `deny` | commits on a detached HEAD not allowed |

Any other `NAME`, such as `head` or `feat/HEAD`, is a branch name, linted as
below.

### Protected branches

A `NAME` in the protected branches fails before its format is checked.

With protected branches `main`:

| `NAME` | Result |
|---|---|
| `main` | direct commits not allowed |
| `feat/main` | linted as below |

### Format

`<type>/<name>`, where `<type>` is one of the allowed types and `<name>`
matches `[a-z][a-z0-9]*(-[a-z0-9]+)*`.

With allowed types `feat`, `fix`:

| `NAME` | Result |
|---|---|
| `feat/add-matcher` | OK |
| `fix/v2` | OK |
| `add-matcher` | missing type prefix: no `/` |
| `feature/add-matcher` | unknown type `feature` |
| `feat/Add-matcher` | invalid name: uppercase letter |
| `feat/add_matcher` | invalid name: `_` |
| `feat/2-add` | invalid name: starts with a digit |
| `feat/add--matcher`, `feat/add-` | invalid name: empty word between `-` |
| `feat/add/matcher` | invalid name: second `/` |
| `feature/Add_Matcher` | unknown type and invalid name, reported together |
