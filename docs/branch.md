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

The first run succeeds silently; the others exit with status `1`.

## Arguments

- `<NAME>`: the branch name to lint, such as the output of
  `git branch --show-current`.

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

When `NAME` is one of the `--protected` branches, the lint fails with advice
to create a branch and open a pull request. The format below is not checked in
that case.

### Format

Otherwise, `NAME` must have the form `<type>/<name>`:

- `<type>` is one of the `--type` values.
- `<name>` is lowercase kebab-case starting with a letter:
  `[a-z][a-z0-9]*(-[a-z0-9]+)*`. It cannot contain another `/`.

An unknown type and an invalid name are reported together. A name without `/`
is reported as missing its type prefix.
