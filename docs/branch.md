# `vcs-lint branch`

Lints a branch name.

```
vcs-lint branch [OPTIONS] <NAME>
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
```

As a Git `pre-commit` hook, here given the name printed by
`git branch --show-current`. Any command that prints the branch name can be
used instead; see [Current branch](#current-branch) for what such commands
give when there is no branch.

```sh
#!/bin/sh
exec vcs-lint branch "$(git branch --show-current)" --protected main
```

## Arguments

- `<NAME>`: the branch name to lint, used as given. vcs-lint does not ask Git
  for the current branch.

## Options

- `--type <TYPE>`: gives the [allowed types](#definitions). A
  [list option](cli.md#list-options).
- `--protected <BRANCH>`: gives the [protected branches](#definitions). A
  [list option](cli.md#list-options).
- `-h`, `--help`: print help.

## Rules

### Definitions

- **Allowed types**: the `--type` values, or `feat`, `fix`, `docs`, `style`,
  `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert` when none are
  given. The same set can be given to
  [`vcs-lint commit`](commit.md#definitions), but nothing requires it.
- **Protected branches**: the `--protected` values. Empty when none are given.

Values are matched exactly, including case.

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

## Current branch

On a detached HEAD, such as after `git checkout --detach`, there is no current
branch, and common commands print:

| Command | Output on a detached HEAD |
|---|---|
| `git branch --show-current` | an empty string |
| `git rev-parse --abbrev-ref HEAD` | `HEAD` |

Neither has a `/`, so either fails as a missing type prefix:

```
$ vcs-lint branch "$(git branch --show-current)"
error: invalid branch name ``
  missing type prefix
    expected: <type>/<name>
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

$ vcs-lint branch "$(git rev-parse --abbrev-ref HEAD)"
error: invalid branch name `HEAD`
  missing type prefix
    expected: <type>/<name>
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
```

A command substitution is also empty when the command itself fails, such as
outside a repository. The shell still runs vcs-lint, which then reports the
empty name after Git's own error.
