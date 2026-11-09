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

## Output

Violations and errors are written to standard error. Nothing is written on
success.

## Exit status

| Status | Meaning |
|---|---|
| `0` | No violation, or help or the version was printed. |
| `1` | A violation was found. |
| `2` | An input could not be read, or the options give rules that cannot be applied, so nothing was linted. |
| `3` | The command line is syntactically invalid. |
