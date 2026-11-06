# `vcs-lint commit`

Lints the header (first line) of a commit message file.

```
vcs-lint commit [OPTIONS] <FILE>
```

## Examples

```
$ cat COMMIT_EDITMSG
feat(core): add matcher
$ vcs-lint commit COMMIT_EDITMSG
error: invalid commit header `feat(core): add matcher`
  unexpected scope `core`
    scopes are not used; remove `(core)`
$ vcs-lint commit COMMIT_EDITMSG --scope root,core,cli

$ cat COMMIT_EDITMSG
feature(parser):add matcher
$ vcs-lint commit COMMIT_EDITMSG --scope root,core,cli
error: invalid commit header `feature(parser):add matcher`
  unknown type `feature`
    allowed types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
  unknown scope `parser`
    allowed scopes: root, core, cli
  expected exactly one space after `:`
```

As a Git `commit-msg` hook, which receives the message file as its first
argument. Comments and the `git commit --verbose` diff in it are skipped (see
[Finding the header](#finding-the-header)), so it can be passed as is:

```sh
#!/bin/sh
exec vcs-lint commit "$1" --scope root,$(ls crates | paste -sd, -)
```

## Arguments

- `<FILE>`: the commit message file. It must be UTF-8 text; a file that
  cannot be read fails with status `1`.

## Options

- `--type <TYPE>`: allowed types. A [list option](cli.md#list-options).
  Defaults to `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
  `build`, `ci`, `chore`, `revert`. Pass the same values as to
  [`vcs-lint branch --type`](branch.md#options) to keep branches and commits
  consistent; nothing requires them to match.
- `--scope <SCOPE>`: allowed scopes. A [list option](cli.md#list-options).
  Whether it is given switches the [header](#header) format.
- `-h`, `--help`: print help.

Values are matched exactly, including case.

## Rules

### Finding the header

The first line left after skipping the following is the header:

```
                                                        skipped: leading blank line
# Please enter the commit message for your changes.     skipped: starts with `#`
feat: add matcher                                       header
                                                        not read
Body text.                                              not read
# ------------------------ >8 ------------------------   skipped: this line and all after it
diff --git a/src/lib.rs b/src/lib.rs                    skipped
```

A message with no line left fails with `commit message is empty`.

### Messages accepted as is

| Header | Result |
|---|---|
| `Merge branch 'main' into feat/x` | accepted: starts with `Merge ` |
| `fixup! feat: add matcher` | accepted: also `squash! ` and `amend! ` |
| `Merged x` | linted: no exact prefix |
| `fixup!feat: x` | linted: no space after `!` |
| `Revert "feat: add matcher"` | linted: malformed header |

### Header

Without `--scope`, the header is `<type>: <subject>`. With `--scope`, it is
`<type>(<scope>): <subject>`. In both, `<type>` is a `--type` value, `:` is
followed by exactly one space, and `<subject>` is not empty.

With `--type feat,fix` and no `--scope`:

| Header | Result |
|---|---|
| `feat: add matcher` | OK |
| `feat(core): add matcher` | unexpected scope `core` |
| `feature: add matcher` | unknown type `feature` |
| `feat:add matcher` | not exactly one space after `:` |
| `feat:  add matcher` | not exactly one space after `:` |
| `feat: ` | empty subject |
| `add matcher` | malformed header |
| `feat!: add matcher` | malformed header: `!` is not recognized |

With `--type feat,fix --scope root,core`:

| Header | Result |
|---|---|
| `feat(core): add matcher` | OK |
| `feat: add matcher` | missing scope |
| `feat(parser): add matcher` | unknown scope `parser` |
| `feat(): add matcher` | unknown scope (empty) |
| `feature(parser):add matcher` | unknown type, unknown scope and spacing, reported together |
