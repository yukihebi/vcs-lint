//! Command line definition, resolved into a [`Lint`].
//!
//! List options accept both repeated flags and comma-separated values, so a
//! list can be written by hand or assembled by a shell.

use std::{fs, io, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::{
    Lint,
    rules::{
        BranchRules, CommitRules, protected::ProtectedRule, scopes::ScopeRule, types::TypeRule,
    },
};

#[derive(Debug, Parser)]
#[command(
    name = "vcs-lint",
    bin_name = "vcs-lint",
    version,
    about,
    no_binary_name = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    /// Builds the rules and reads the input the command line refers to.
    pub fn into_lint(self) -> Result<Lint, ReadError> {
        match self.command {
            Command::Branch(args) => Ok(args.into_lint()),
            Command::Commit(args) => args.into_lint(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Lint a branch name of the form `<type>/<name>`
    Branch(BranchArgs),
    /// Lint the header of a commit message file
    Commit(CommitArgs),
}

#[derive(Debug, Args)]
struct BranchArgs {
    /// Branch name to lint
    name: String,
    #[command(flatten)]
    types: TypeArgs,
    /// Branch that must not be committed to directly, given as repeated flags
    /// or comma-separated values
    #[arg(long = "protected", value_name = "BRANCH", value_delimiter = ',')]
    protected: Vec<String>,
}

impl BranchArgs {
    fn into_lint(self) -> Lint {
        Lint::Branch {
            name: self.name,
            rules: BranchRules {
                types: self.types.into_rule(),
                protected: ProtectedRule::new(self.protected.into_iter().collect()),
            },
        }
    }
}

#[derive(Debug, Args)]
struct CommitArgs {
    /// Commit message file, such as the one given to the commit-msg hook
    file: PathBuf,
    #[command(flatten)]
    types: TypeArgs,
    /// Scope that commit headers must have, given as repeated flags or
    /// comma-separated values; without any, headers must not have a scope
    #[arg(long = "scope", value_name = "SCOPE", value_delimiter = ',')]
    scopes: Vec<String>,
}

impl CommitArgs {
    fn into_lint(self) -> Result<Lint, ReadError> {
        let Self {
            file,
            types,
            scopes,
        } = self;
        let message = fs::read_to_string(&file).map_err(|source| ReadError { file, source })?;
        Ok(Lint::Commit {
            message,
            rules: CommitRules {
                types: types.into_rule(),
                scopes: ScopeRule::new(scopes.into_iter().collect()),
            },
        })
    }
}

#[derive(Debug, Args)]
struct TypeArgs {
    #[arg(
        long = "type",
        value_name = "TYPE",
        value_delimiter = ',',
        help = TypeArgs::help(),
    )]
    types: Vec<String>,
}

impl TypeArgs {
    fn help() -> String {
        format!(
            "Allowed type, given as repeated flags or comma-separated values\n[default: {}]",
            TypeRule::DEFAULT.join(", ")
        )
    }

    fn into_rule(self) -> TypeRule {
        TypeRule::new(self.types.into_iter().collect())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("cannot read commit message file `{}`\n  {source}", file.display())]
pub struct ReadError {
    file: PathBuf,
    source: io::Error,
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
