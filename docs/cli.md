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
flag valid when the substitution is empty.

### Composition

Each list option comes with an exclusion option, named after it with
`-exclude`, such as `--scope-exclude` for `--scope`. An exclusion option is a
list option itself.

The values of the list are composed as follows, whatever the order of the
options:

1. The defaults of the list, when they are [included](#defaults).
2. Then the values given to the list option, without repeating a value.
3. Without the values given to the exclusion option.

Values count as given once the list option appears; the exclusion option
alone does not give values. Each command decides what an empty list means.

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

## Output

Violations and errors are written to standard error. Nothing is written on
success.

## Exit status

| Status | Meaning |
|---|---|
| `0` | No violation, or help or the version was printed. |
| `1` | A violation was found. |
| `2` | A runtime error occurred, so nothing was linted. |
| `3` | The command line is syntactically invalid. |
