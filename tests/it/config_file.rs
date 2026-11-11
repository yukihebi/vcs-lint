//! Options read from the config file given with `--config` or
//! `VCS_LINT_CONFIG`.

use std::io::Write;

use insta::assert_snapshot;
use tempfile::NamedTempFile;

use vcs_lint::RunStatus;

use super::{BinaryOutput, Run, workdir::Workdir};

const VAR: &str = "VCS_LINT_CONFIG";

/// A config file holding `text`.
fn config(text: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(text.as_bytes()).unwrap();
    file
}

fn path(file: &NamedTempFile) -> &str {
    file.path().to_str().unwrap()
}

/// A repository with a commit message file `MSG` holding `message`.
fn repo(message: &str) -> Workdir {
    let repo = Workdir::repo("feat/x");
    repo.write("MSG", message);
    repo
}

#[test]
fn reads_options_from_file_given_with_option() {
    let file = config("[branch]\nprotected = [\"main\"]\n");
    let run = Run::new(&["branch", "main", "--config", path(&file)]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: branch `main` is protected
    ");
}

#[test]
fn reads_options_from_file_named_by_variable() {
    let file = config("type = [\"feat\"]\n");
    let run = Run::with_vars(&["branch", "fix/x"], &[(VAR, path(&file))]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `fix/x`
      unknown type `fix`
        allowed types: feat
    ");
}

#[test]
fn reads_file_given_with_option_instead_of_variable() {
    let given = config("[branch]\nprotected = [\"main\"]\n");
    let named = config("scopes = []\n");
    let run = Run::with_vars(
        &["branch", "main", "--config", path(&given)],
        &[(VAR, path(&named))],
    );
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_eq!(run.output.code, Some(1));
}

#[test]
fn reads_no_file_when_variable_is_empty() {
    let run = Run::with_vars(&["branch", "feat/x"], &[(VAR, "")]);
    assert_status!(run, RunStatus::Success);
    assert_eq!(run.output.code, Some(0));
}

#[test]
fn reads_shared_and_own_keys_before_command_line() {
    let file = config(
        "type = [\"feat\"]\nscope = [\"root\"]\n[commit]\nscope = [\"cli\"]\nscope-exclude = [\"core\"]\n",
    );
    let run = Run::commit(
        "fix(x): add matcher\n",
        &["--config", path(&file), "--scope", "core,app"],
    );
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `fix(x): add matcher`
      unknown type `fix`
        allowed types: feat
      unknown scope `x`
        allowed scopes: root, cli, app
    ");
}

#[test]
fn reads_path_relative_to_current_directory() {
    let repo = repo("feat: add matcher\n");
    repo.write(
        "conf/vcs-lint.toml",
        "[commit]\nscope-mode = \"required\"\nscope = [\"core\"]\n",
    );
    let given = repo.run(&["commit", "MSG", "--config", "conf/vcs-lint.toml"]);
    let named = BinaryOutput::of(
        repo.vcs_lint(&["commit", "MSG"])
            .env(VAR, "conf/vcs-lint.toml"),
    );
    assert_eq!(given.to_string(), named.to_string());
    assert_snapshot!(given, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `feat: add matcher`
      missing scope
        allowed scopes: core
    ");
}

#[test]
fn fails_on_options_given_once_in_file_and_command_line() {
    let file = config("detached = \"deny\"\n");
    let run = Run::new(&[
        "branch",
        "HEAD",
        "--config",
        path(&file),
        "--detached",
        "allow",
    ]);
    assert_status!(run, RunStatus::ConfigFileError(_));
    assert_eq!(run.output.code, Some(2));

    let repo = repo("feat: add matcher\n");
    repo.write(
        "vcs-lint.toml",
        "type-defaults = \"never\"\ntype = [\"feat\"]\n[commit]\nscope-mode = \"required\"\n",
    );
    let args = [
        "commit",
        "MSG",
        "--config",
        "vcs-lint.toml",
        "--scope-mode",
        "optional",
        "--type-defaults=always",
        "--type",
        "fix",
    ];
    assert_snapshot!(repo.run(&args), @"
    exit: 2
    --- stdout
    --- stderr
    error: config file `vcs-lint.toml` and the command line both give options that can be given once
      `type-defaults` and `--type-defaults`
      `commit.scope-mode` and `--scope-mode`
    ");
}

#[test]
fn fails_on_values_command_line_rejects() {
    let file = config("[branch]\ndetached = \"Deny\"\n");
    let run = Run::new(&["branch", "HEAD", "--config", path(&file)]);
    assert_status!(run, RunStatus::ConfigFileError(_));
    assert_eq!(run.output.code, Some(2));

    let repo = repo("feat: add matcher\n");
    repo.write(
        "vcs-lint.toml",
        "scope = [\"core\", \" , \"]\n[branch]\ndetached = \"Deny\"\n[commit]\nscope-mode = \"\"\n",
    );
    assert_snapshot!(repo.run(&["branch", "--config", "vcs-lint.toml"]), @r#"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      invalid value " , " of `scope`
        every value is empty
      invalid value "Deny" of `branch.detached`
        expected `allow` or `deny`
      invalid value "" of `commit.scope-mode`
        expected `auto`, `required`, `optional` or `forbidden`
    "#);
}

#[test]
fn fails_on_option_given_once_at_both_levels() {
    let repo = repo("feat: add matcher\n");
    repo.write(
        "vcs-lint.toml",
        "detached = \"deny\"\n[branch]\ndetached = \"allow\"\n",
    );
    assert_snapshot!(repo.run(&["commit", "MSG", "--config", "vcs-lint.toml"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      repeated key `branch.detached`
        `detached` at the top level gives the same option, which can be given once
    ");
}

#[test]
fn fails_on_missing_file() {
    let repo = repo("feat: add matcher\n");
    assert_snapshot!(repo.run(&["commit", "MSG", "--config", "vcs-lint.toml"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: cannot read config file `vcs-lint.toml`
      No such file or directory (os error 2)
    ");
}

#[test]
fn fails_on_file_other_than_toml() {
    let repo = repo("feat: add matcher\n");
    repo.write("vcs-lint.toml", "[commit\nscope = [\"core\"]\n");
    assert_snapshot!(repo.run(&["commit", "MSG", "--config", "vcs-lint.toml"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      TOML parse error at line 1, column 8
        |
      1 | [commit
        |        ^
      unclosed table, expected `]`
    ");
}

#[test]
fn fails_on_every_invalid_key_whichever_command_runs() {
    let repo = repo("feat: add matcher\n");
    repo.write(
        "vcs-lint.toml",
        "scopes = [\"core\"]\nconfig = \"other.toml\"\ntype = \"feat\"\n\n\
         [branch]\nscope = [\"core\"]\n\"detached HEAD\" = \"deny\"\n\n\
         [commit]\nscope-mode = [\"required\"]\n",
    );
    let commit = repo.run(&["commit", "MSG", "--config", "vcs-lint.toml"]);
    let branch = repo.run(&["branch", "--config", "vcs-lint.toml"]);
    assert_eq!(commit.to_string(), branch.to_string());
    assert_snapshot!(commit, @r#"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      unknown key `scopes`
        no command has an option of this name
      disallowed key `config`
        this option cannot be given in a config file
      invalid value of `type`
        expected an array of strings
      unknown key `branch.scope`
        `vcs-lint branch` has no option of this name
      unknown key `branch."detached HEAD"`
        `vcs-lint branch` has no option of this name
      invalid value of `commit.scope-mode`
        expected a string
    "#);
}

#[test]
fn fails_on_command_key_other_than_table() {
    let repo = repo("feat: add matcher\n");
    repo.write("vcs-lint.toml", "commit = \"MSG\"\n");
    assert_snapshot!(repo.run(&["commit", "MSG", "--config", "vcs-lint.toml"]), @"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      invalid value of `commit`
        expected a table of the options of `vcs-lint commit`
    ");
}

#[test]
fn reads_no_file_for_help() {
    let run = Run::with_vars(
        &["branch", "--help", "--config", "missing.toml"],
        &[(VAR, "missing.toml")],
    );
    assert_status!(run, RunStatus::Info(_));
    assert_eq!(run.output.code, Some(0));
}

#[test]
fn reads_no_file_when_command_line_is_invalid() {
    let run = Run::with_vars(&["commit"], &[(VAR, "missing.toml")]);
    assert_status!(run, RunStatus::SyntaxError(_));
    assert_eq!(run.output.code, Some(2));
}

#[test]
fn reads_hints_from_top_level_and_tables() {
    let file = config(
        "type-hint = \"types are listed in CONTRIBUTING.md\"\n\
         \n\
         [branch]\n\
         protected = [\"main\"]\n\
         protected-hint = \"\"\"\n\
         update main through a pull request\n\
         \"\"\"\n",
    );
    let run = Run::new(&["branch", "main", "--config", path(&file)]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: branch `main` is protected
    hint: update main through a pull request
    ");
    let run = Run::new(&["branch", "fix/x", "--type", "feat", "--config", path(&file)]);
    assert_status!(run, RunStatus::BranchLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid branch name `fix/x`
      unknown type `fix`
        allowed types: feat
    hint: types are listed in CONTRIBUTING.md
    ");
    let run = Run::commit("fix: x\n", &["--type", "feat", "--config", path(&file)]);
    assert_status!(run, RunStatus::CommitLintError(_));
    assert_snapshot!(run, @"
    exit: 1
    --- stdout
    --- stderr
    error: invalid commit header `fix: x`
      unknown type `fix`
        allowed types: feat
    hint: types are listed in CONTRIBUTING.md
    ");
}

#[test]
fn reports_text_that_is_not_hint_by_its_key() {
    let repo = repo("feat: add matcher\n");
    repo.write(
        "vcs-lint.toml",
        "[branch]\nprotected-hint = \"git branch -m {new}\"\nformat-hint = \"\"\n",
    );
    assert_snapshot!(repo.run(&["branch", "--config", "vcs-lint.toml"]), @r#"
    exit: 2
    --- stdout
    --- stderr
    error: invalid config file `vcs-lint.toml`
      invalid value "git branch -m {new}" of `branch.protected-hint`
        placeholder `{new}` is not supported; write `{{` and `}}` for braces
      invalid value "" of `branch.format-hint`
        the hint is empty
    "#);
}
