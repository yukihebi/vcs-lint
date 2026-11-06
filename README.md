# vcs-lint

A single command line tool that lints branch names and commit message headers.

Allowed types and scopes are passed as options, so the caller (a git hook, a
task runner, a CI step) decides where they come from. A monorepo can pass its
module list as scopes; a single-module repository simply omits them.

## Features

- Lints branch names of the form `<type>/<name>`, and rejects direct commits
  to protected branches such as `main`.
- Lints commit message headers of the form `<type>: <subject>`, or
  `<type>(<scope>): <subject>` when scopes are given.
- Reads the file given to git's `commit-msg` hook as is, skipping git comments
  and accepting merge and autosquash messages.
- Reports every violation at once, with the values that would be accepted.
- Needs no configuration file and no git: everything is passed as arguments.

## Usage

```sh
vcs-lint branch feat/add-matcher --protected main
vcs-lint commit .git/COMMIT_EDITMSG
vcs-lint commit .git/COMMIT_EDITMSG --scope root,$(ls crates | paste -sd, -)
```

```
$ vcs-lint branch add-matcher
error: invalid branch name `add-matcher`
  missing type prefix
    expected: <type>/<name>
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
```

See `vcs-lint --help` for options, and the documents below for the detailed
rules:

- [Command line](docs/cli.md): list options, output and exit status.
- [`vcs-lint branch`](docs/branch.md)
- [`vcs-lint commit`](docs/commit.md)

## License

See the LICENSE file.
