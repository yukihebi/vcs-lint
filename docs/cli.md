# Command line

```
vcs-lint <COMMAND> [ARGS]...
vcs-lint --help
vcs-lint --version
```

## Commands

| Command | Description |
|---|---|
| [`branch`](branch.md) | Lint a branch name. |
| [`commit`](commit.md) | Lint the header of a commit message file. |
| [`history`](history.md) | Lint whether the history of a revision contains the tip of its base branch. |
| `help [COMMAND]` | Print the help of vcs-lint or of a command. |

Each command documents its own arguments and options.

## Options

- `-h`, `--help`: print help. Also accepted by every command
  (`vcs-lint branch --help`).
- `-V`, `--version`: print the version.

## List options

Some command options take a list of values. They all accept the same forms,
which may be mixed:

```sh
vcs-lint commit COMMIT_EDITMSG --scope root --scope cli    # repeated
vcs-lint commit COMMIT_EDITMSG --scope root,cli            # comma-separated
vcs-lint commit COMMIT_EDITMSG --scope root,cli --scope core
```

The values are then normalized:

- Whitespace around each value is trimmed.
- Empty values (`a,,b`, a trailing comma) are dropped.
- A repeated value is kept once, at its first position.

Each flag must be left with at least one value. A flag given only empty
values, such as `--scope ""`, `--scope ,` or a shell substitution that comes
out empty, makes the command line syntactically invalid even when other flags
give values:

```
$ vcs-lint commit COMMIT_EDITMSG --scope root --scope "$(ls crates | paste -sd, -)"
error: invalid value '' for '--scope <SCOPE>': every value is empty
```

Adding a fixed value to the same flag, as in `--scope root,$(…)`, keeps the
flag valid when the substitution is empty. To add the directories under a
directory, use its [directory option](#directories) instead.

### Directories

Each list option comes with a directory option, named after it with `-dir`,
such as `--scope-dir` for `--scope`, which adds the names of the directories
directly under each `DIR` given to it. A directory option is a list option
itself, and each `DIR` is relative to the current directory:

```sh
vcs-lint commit COMMIT_EDITMSG --scope root --scope-dir crates   # root and the packages under crates
```

The directories are read from the Git index, as `git ls-files` lists files: a
directory counts when it holds a committed or staged file at any depth, and a
submodule counts as a directory. A package added in the commit being linted is
then included, while untracked directories, such as build outputs, are not.
Hidden directories such as `.github` count as well; exclude unwanted ones with
the [exclusion option](#composition). The names are used as they are, ordered
by their bytes.

A `DIR` that does not exist, is not a directory, or is outside the work tree of
the Git repository found from the current directory fails with status `2`:

```
$ vcs-lint commit COMMIT_EDITMSG --scope-dir packages
error: cannot read the directories under `packages` for `--scope-dir`
  no such directory
```

Each `DIR` must add at least one directory. A `DIR` with none, such as one
holding only files or only untracked directories, fails with status `2` as
well, even when other values are given:

```
$ vcs-lint commit COMMIT_EDITMSG --scope root --scope-dir crates/core/src
error: there are no directories under `crates/core/src` for `--scope-dir`
  a directory counts when it holds a committed or staged file
```

### Composition

Each list option comes with an exclusion option, named after it with
`-exclude`, such as `--scope-exclude` for `--scope`. An exclusion option is a
list option itself.

The values of the list are composed as follows, whatever the order of the
options:

1. The defaults of the list, when they are [included](#defaults).
2. Then the values given to the list option, then the names of the directories
   given to its [directory option](#directories), without repeating a value.
3. Without the values given to the exclusion option.

Values count as given once the list option or its directory option appears,
even if every value is excluded; the exclusion option alone does not give
values.
Each command decides what an empty list means.

Excluding a value that is not in the list fails with status `2`:

```
$ vcs-lint commit COMMIT_EDITMSG --scope root,cli --scope-exclude core
error: cannot exclude values outside the allowed scopes: core
  allowed scopes: root, cli
```

### Values nothing can match

Once composed, a list must not hold a value that nothing linted could match,
such as a type that no commit header can have. Which values these are depends
on the command and the list; a value with a line break is one of them for
every list, as what is linted is a single line.

Such values fail with status `2`, for every list holding some at once, and
values are neither converted nor dropped:

```
$ vcs-lint commit COMMIT_EDITMSG --type feat,wip! --scope 'core,core(unix'
error: no commit header can match these values
  allowed types: "wip!"
    a type cannot contain `(`, `)`, `:`, `!` or whitespace, nor start with `#`
  allowed scopes: "core(unix"
    a scope cannot contain a line break, and its parentheses must be balanced
```

Only values left in the list are checked, so excluding a value takes it out of
the check.

### Defaults

A list option with defaults comes with an option named after it with
`-defaults`, which decides when the defaults are included:

| Mode | Defaults |
|---|---|
| `auto`, or not given | included only when no values are given |
| `always` | always included |
| `never` | never included |

Only `--type` has defaults, so `--type-defaults` is the only such option:

```sh
vcs-lint commit COMMIT_EDITMSG --type-exclude chore              # the defaults but chore
vcs-lint commit COMMIT_EDITMSG --type wip --type-defaults always # the defaults and wip
vcs-lint commit COMMIT_EDITMSG --type wip                        # wip alone
```

## Config file

Every command reads options from the TOML file given with
`--config <PATH>`, or from the one [`VCS_LINT_CONFIG`](#vcs_lint_config) names
when `--config` is not given, so every place that runs vcs-lint, such as hooks
and CI, can share the same rules. `PATH` is relative to the current directory.
No file is read unless one is given.

```toml
type = ["feat", "fix", "docs"]

[branch]
protected = ["main"]

[commit]
scope = ["root"]
scope-mode = "required"

[history]
base = "main"
```

### Keys

A key is the long name of an option, without `--`, written at the top level or
in a table named after a command, such as `[branch]`, each optional:

- A key at the top level is read by every command that has the option, and
  ignored by the others: `type` by `branch` and `commit`, `scope` by `commit`
  only.
- A key in a table is read by its command only.

A [list option](#list-options) is an array of strings, each counting as the
value of one flag: `scope = ["root", "cli"]` means `--scope root --scope cli`,
and `scope = []` gives no flag. Any other option is a string, such as
`scope-mode = "required"`.

The arguments, such as `NAME`, `FILE` and `REV`, `--help` and `--config`
cannot be written.

### Reading options

The keys a command reads, those at the top level and then those in its table,
mean the same as their options given in that order at the start of the command
line. The rules of the command line apply to them as they are, so the command
line adds to the file:

```
$ vcs-lint commit COMMIT_EDITMSG --config vcs-lint.toml --scope cli   # root and cli
```

An option other than a list option can be given once, so giving it both in the
file and on the command line fails with status `2`, for every such option at
once:

```
$ vcs-lint commit COMMIT_EDITMSG --config vcs-lint.toml --scope-mode optional
error: config file `vcs-lint.toml` and the command line both give options that can be given once
  `commit.scope-mode` and `--scope-mode`
```

### Invalid files

A file that does not exist, cannot be read or is not TOML fails with status
`2`, as does a file holding an invalid key:

- A key at the top level that no command has, or a key in a table that its
  command does not have.
- A value that is not of the type of its option, or a string the command line
  rejects as the value of the option, such as `scope = [","]` or
  `scope-mode = "bogus"`.
- A key in a table giving an option other than a list option that the command
  reads at the top level as well.

Every key is checked whichever command runs, so a file is valid for every
command or for none, and every invalid key is reported at once:

```
$ vcs-lint commit COMMIT_EDITMSG --config vcs-lint.toml
error: invalid config file `vcs-lint.toml`
  unknown key `branch.scope`
    `vcs-lint branch` has no option of this name
  invalid value "bogus" of `commit.scope-mode`
    expected `auto`, `required`, `optional` or `forbidden`
```

The file is read once the rest of the command line is found valid, so help,
the version and a syntactically invalid command line are printed without
reading it.

## Environment variables

### `VCS_LINT_REQUIRED_VERSION`

The minimum version of vcs-lint required, such as `0.2.0`. When the running
vcs-lint is older, every command fails with status `3` before the command line
is read, `--help` and `--version` included, as an older vcs-lint may not know
the options the command line uses:

```
$ VCS_LINT_REQUIRED_VERSION=0.2.0 vcs-lint commit COMMIT_EDITMSG --scope-dir crates
error: vcs-lint 0.2.0 or later is required, but this is 0.1.0-alpha.6
  required by `VCS_LINT_REQUIRED_VERSION`
```

A version is written `MAJOR.MINOR.PATCH`, optionally followed by `-alpha.N`,
`-beta.N` or `-rc.N`, with numbers in decimal without leading zeros. Versions
are ordered by SemVer precedence:
`0.1.0-alpha.6` < `0.1.0-alpha.10` < `0.1.0-beta.0` < `0.1.0-rc.1` < `0.1.0`.
Build metadata such as `+build` takes no part in the order, so it is not
accepted.

Whitespace around the value is trimmed, and a blank value requires nothing. A
value that is not a version fails with status `3` as well:

```
$ VCS_LINT_REQUIRED_VERSION=v0.2 vcs-lint commit COMMIT_EDITMSG
error: invalid version "v0.2" in `VCS_LINT_REQUIRED_VERSION`
  expected `MAJOR.MINOR.PATCH`, optionally followed by `-alpha.N`, `-beta.N` or `-rc.N`
```

vcs-lint released before this variable ignores it.

### `VCS_LINT_CONFIG`

The [config file](#config-file) that every command reads when `--config` is
not given, relative to the current directory. It is read by every command, as
the file has a table for each, so it can be set once for every place that runs
vcs-lint:

```sh
export VCS_LINT_CONFIG=vcs-lint.toml
```

An empty value counts as unset. vcs-lint released before this variable ignores
it.

## Output

Violations and errors are written to standard error. Nothing is written on
success.

### Severity

Every command takes `--severity <LEVEL>`, which decides how violations are
reported:

| Level | Violations |
|---|---|
| `error`, or not given | reported as errors, failing with status `1` |
| `warning` | reported as warnings, exiting with status `0` |

A warning is worded as the error would be, starting with `warning:` instead, so
a rule can be introduced without blocking anything until it is followed:

```
$ vcs-lint branch main --protected main --severity warning
warning: branch `main` is protected
```

Only violations change. Errors keep their status, as nothing was linted, and
passing on them would hide that the check did not run.

## Exit status

| Status | Meaning |
|---|---|
| `0` | No violation, violations were reported as [warnings](#severity), or help or the version was printed. |
| `1` | A violation was found and reported as an error. |
| `2` | An error occurred, such as an invalid command line or a [config file](#config-file) that cannot be read, so nothing was linted. |
| `3` | The running vcs-lint is older than the [required version](#vcs_lint_required_version), or the required version is invalid, so nothing was run. |
