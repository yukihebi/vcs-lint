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

As a Git `pre-commit` hook, linting the current branch:

```sh
#!/bin/sh
exec vcs-lint branch "$(git branch --show-current)" --protected main
```

## Arguments

- `<NAME>`: the branch name to lint.

## Options

- `--type <TYPE>`: allowed types. A [list option](cli.md#list-options).
  Defaults to `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
  `build`, `ci`, `chore`, `revert`. Pass the same values as to
  [`vcs-lint commit --type`](commit.md#options) to keep branches and commits
  consistent; nothing requires them to match.
- `--protected <BRANCH>`: branches that must not receive direct commits. A
  [list option](cli.md#list-options). Nothing is protected by default.
- `-h`, `--help`: print help.

Values are matched exactly, including case.

## Rules

### Protected branches

A protected branch fails before its format is checked.

| Command | Result |
|---|---|
| `vcs-lint branch main --protected main` | direct commits not allowed |
| `vcs-lint branch feat/main --protected main` | linted as below |

### Format

`<type>/<name>`, where `<type>` is a `--type` value and `<name>` matches
`[a-z][a-z0-9]*(-[a-z0-9]+)*`.

With `--type feat,fix`:

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
