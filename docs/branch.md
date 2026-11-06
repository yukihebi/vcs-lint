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

As a Git `pre-commit` hook, linting the current branch.
`git branch --show-current` is used here for brevity but is not recommended:
it prints nothing both on a detached HEAD and when it fails, so the two cannot
be told apart.

```sh
#!/bin/sh
exec vcs-lint branch "$(git branch --show-current)" --protected main
```

## Arguments

- `<NAME>`: the branch name to lint.

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
