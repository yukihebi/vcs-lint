# vcs-lint

A Simple, Fast Linter for Commit Messages and Branch Names in Version Control Systems

## Features

- All rules can be passed as command line arguments.
- The file path from Git's `commit-msg` hook can be passed as-is.
- Lints branch names of the form `<type>/<name>`,
  rejects direct commits to protected branches such as `main`,
  and can reject commits on a detached HEAD.
- Lints commit message headers of the form `<type>: <subject>`,
  or `<type>(<scope>): <subject>`.
- Reports every violation at once.
- Failure can be detected by the exit status.

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

See `vcs-lint --help` for options, and the documents below for the detailed rules:

- [Command line](docs/cli.md): commands, list options, output and exit status.
- [`vcs-lint branch`](docs/branch.md)
- [`vcs-lint commit`](docs/commit.md)

## License

See the LICENSE file.
