# vcs-lint

A Simple, Fast Linter for Commit Messages and Branch Names in Version Control Systems

## Features

- All rules can be passed as command line arguments,
  or shared through a config file read by every hook and CI job.
- The file path from Git's `commit-msg` hook can be passed as-is.
- Lints branch names of the form `<type>/<name>`,
  rejects protected branches such as `main`,
  and can reject a detached HEAD.
  The current branch is read from Git when no name is given.
- Lints commit message headers of the form `<type>: <subject>`,
  or `<type>(<scope>): <subject>`.
- Reports every violation at once.
- Failure can be detected by the exit status.

## Installation

Build and install with the [Rust](https://rust-lang.org/) toolchain:

```sh
cargo install --locked --git https://github.com/yukihebi/vcs-lint
```

## Usage

```sh
vcs-lint branch --protected main
vcs-lint branch feat/add-matcher --protected main
vcs-lint commit .git/COMMIT_EDITMSG
vcs-lint commit .git/COMMIT_EDITMSG --scope root --scope-dir crates
vcs-lint commit .git/COMMIT_EDITMSG --config vcs-lint.toml
```

```
$ vcs-lint branch add-matcher
error: invalid branch name `add-matcher`
  missing type prefix
    expected: <type>/<name>
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
```

See `vcs-lint --help` for options, and the documents below for the detailed rules:

- [Command line](docs/cli.md): commands, list options, config file, environment variables, output and exit status.
- [`vcs-lint branch`](docs/branch.md)
- [`vcs-lint commit`](docs/commit.md)

## License

See the LICENSE file.
