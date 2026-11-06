# `vcs-lint branch`

```
vcs-lint branch <NAME> [--type <TYPE>]... [--protected <BRANCH>]...
```

Lints the branch name `NAME`. List options are written as described in
[cli.md](cli.md#list-options).

## Format

A branch name has the form `<type>/<name>`:

- `<type>` is one of the allowed types (`--type`).
- `<name>` is lowercase kebab-case starting with a letter:
  `[a-z][a-z0-9]*(-[a-z0-9]+)*`. It cannot contain another `/`.

A missing type prefix, an unknown type and an invalid name are reported
together.

## Protected branches

`--protected <BRANCH>` names branches that must not receive direct commits,
such as `main`. Nothing is protected by default.

A protected branch fails with advice to create a branch and open a pull
request. Its format is not checked, since renaming it is not the fix:

```
error: direct commits to `main` are not allowed
  create a branch and open a pull request instead
```
