# `vcs-lint history`

Lints whether the history of a revision contains the tip of its base branch,
so that a branch that has fallen behind its base is found before a pull
request is opened, as where branches must be up to date before merging.

```
vcs-lint history [OPTIONS] --base <BRANCH> [REV]
```

## Examples

```
$ vcs-lint history --base main

$ vcs-lint history --base main
error: `HEAD` is behind base branch `main` on `origin`
  missing 2 commits of the base

$ vcs-lint history feat/add-matcher --base main --network deny
error: `feat/add-matcher` is behind base branch `origin/main`
  missing 2 commits of the base
```

As a Git `pre-push` hook, linting the current commit against `main` on the
remote:

```sh
#!/bin/sh
exec vcs-lint history --base main
```

In CI, where the commit is often checked out by its hash, the base can be
fetched first and read from its remote-tracking branch:

```sh
git fetch origin main
vcs-lint history "$COMMIT" --base main --network deny
```

A shallow clone, as CI often makes, must hold enough history to find the tip
(see [Errors](#errors)).

## Arguments

- `[REV]`: the revision to lint, in any form Git reads, such as a branch
  name, `HEAD~1`, a commit hash, a tag or `:/<text>`. `HEAD` when not given.
  Only the commit it names matters, so a branch and a detached HEAD at the
  same commit are linted alike, and a tag is linted as the commit it points
  to.

## Options

- `--base <BRANCH>`: the base branch, by its name without `refs/heads/`.
  Required, on the command line or in a [config file](cli.md#config-file).
  Git has no notion of the base of a branch, so it is never inferred.
- `--base-from <SOURCE>`: where the [tip](#tip-of-the-base) is read from,
  `local` or `remote`. `remote` when not given.
- `--remote <NAME>`: the remote the base is read from with
  `--base-from remote`. `origin` when not given.
- `--network <ACCESS>`: whether vcs-lint may access the network, `allow` or
  `deny`. `allow` when not given.
- `--severity <LEVEL>`: how violations are reported, `error` or `warning`
  (see [Severity](cli.md#severity)). `error` when not given.
- `--config <PATH>`: reads options from a
  [config file](cli.md#config-file). The file named by
  [`VCS_LINT_CONFIG`](cli.md#vcs_lint_config) when not given.
- `-h`, `--help`: print help.

`--remote` and `--network` are used only where the base is read from a
remote, and are ignored otherwise, so they can be shared through a config
file.

## Tip of the base

The tip is read from Git, run as a child process in the current directory,
from the ref that `--base-from` and `--network` select:

| `--base-from` | `--network allow` | `--network deny` |
|---|---|---|
| `local` | `refs/heads/<BRANCH>` | `refs/heads/<BRANCH>` |
| `remote`, or not given | `refs/heads/<BRANCH>` on the remote, read with `git ls-remote` | the remote-tracking branch `refs/remotes/<NAME>/<BRANCH>` |

- A local base is often behind the remote itself, so `local` is the weaker
  check.
- `git ls-remote` reads the remote as it is now. It runs without a time
  limit, so an unreachable remote holds the command until the connection
  times out.
- The remote-tracking branch is the tip as last fetched, so it may be older
  than the remote.

### Network

Only `git ls-remote` accesses the network. A partial clone does not fetch the
objects it lacks either, so a commit it lacks counts as missing from the
repository. Git released before May 2024 cannot be kept from fetching them,
and fetches them from the remote it was cloned from.

Git runs with prompts for credentials turned off, so it fails instead of
waiting for input, whether on the terminal (`GIT_TERMINAL_PROMPT=0`) or with
a program (`GIT_ASKPASS` set empty, and `SSH_ASKPASS_REQUIRE=never` for SSH).
SSH itself may still ask on the terminal, such as for a passphrase or an
unknown host key. In a hook that must never wait, run SSH in batch mode:

```sh
GIT_SSH_COMMAND="ssh -o BatchMode=yes" vcs-lint history --base main
```

## Rules

`REV` passes when its history contains the tip, that is, when the tip is
`REV` itself or one of its ancestors. A branch updated with its base passes
whether it was merged or rebased.

| History of `REV` | Result |
|---|---|
| contains the tip | OK |
| does not contain the tip | behind, by the number of commits of the base it is missing |
| the tip, read with `git ls-remote`, is not in the local repository | behind, without a number, as `REV` cannot contain a commit the repository lacks |

```
$ vcs-lint history --base main
error: `HEAD` is behind base branch `main` on `origin`
  the tip 3f1c2a9e0b7d4c6a8e5f1b2d3c4a5e6f7a8b9c0d is not in the local repository
    fetch it to see how many commits are missing
```

This checks whether the history is up to date with the base, not whether it
conflicts with it: a branch behind its base often merges cleanly.

## Errors

Nothing is linted, and the command fails with status `2`, when:

- `--base` is not given.
- `--base` cannot be a branch name, such as `main~1`, or the remote name makes
  the remote-tracking branch an invalid ref name.
- Git is not found, or no repository is found from the current directory.
- `REV` cannot be resolved to a commit.
- The base does not exist where it is read from.
- `git ls-remote` fails, such as when the remote cannot be reached. It never
  falls back to the remote-tracking branch, which would give an older tip
  without saying so.
- The repository is shallow and the tip is not found in the history of `REV`,
  whether it is not in the repository or not among the ancestors it has, as
  the history it is missing may contain the tip.

```
$ vcs-lint history --base main
error: cannot tell whether `HEAD` contains the tip of base branch `main` on `origin`
  the tip is not found in the history of a shallow repository, which may be missing commits
  fetch more history, e.g. with `git fetch --unshallow`
```

A shallow repository passes when the tip is found in the history it has.
