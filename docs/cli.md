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
- A list left without values counts as an omitted option.

## Output

Violations and errors are written to standard error. Nothing is written on
success.

## Exit status

| Status | Meaning |
|---|---|
| `0` | No violation. |
| `1` | A violation was found, or an input could not be read. |
| `2` | The command line is invalid, such as an unknown option, a missing argument or an argument that is not valid UTF-8. |
