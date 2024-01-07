use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Bump and update the history file.
    Bump(BumpArgs),
    /// View the history file.
    /// Blocks on stdin if --augment and/or --restrict are passed
    View(ViewArgs),
    /// Delete an entry from the history file.
    Delete(DeleteArgs),
}

#[derive(Debug, clap::Args)]
pub struct BumpArgs {
    /// History file path
    pub path: PathBuf,

    /// Entry to bump.
    /// If omitted, this acts like a touch; the regular score updates are performed, and the file is created if it didn't already exist, but no entries are bumped.
    pub key: Option<String>,

    /// Expiration threshold.
    /// Entries with a monthly energy below this threshold are pured from the file.
    #[arg(short, long, default_value_t = 0.1, value_name = "NUM")]
    pub threshold: f64,

    /// Throw an error if the target file does not exist, instead of createing a new file.
    #[arg(short = 'e', long = "strict", value_name = "NUM")]
    pub strict: bool,
}

#[derive(Debug, clap::Args)]
pub struct ViewArgs {
    /// History file path
    pub path: PathBuf,

    /// Augment the output with entries read from stdin.
    ///
    /// If they're not already present in the history file, they are appended to the bottom of
    /// the output.
    /// Entries should be newline-separated.
    #[arg(short, long)]
    pub augment: bool,
    /// Restrict the output to entries read from stdin.
    ///
    /// Entries should be newline-separated.
    #[arg(short, long)]
    pub restrict: bool,

    /// Hourly energy weight
    #[arg(short = 'u', long, default_value_t = 400.0, value_name = "NUM")]
    pub hourly: f64,
    /// Daily energy weight
    #[arg(short, long, default_value_t = 20.0, value_name = "NUM")]
    pub daily: f64,
    /// Monthly energy weight
    #[arg(short, long, default_value_t = 1.0, value_name = "NUM")]
    pub monthly: f64,

    /// Dump a list of all scores.
    #[arg(long)]
    pub scores: bool,

    /// Throw an error if the target file does not exist, instead of createing a new file.
    #[arg(short, long, value_name = "NUM")]
    pub strict: bool,
}

#[derive(Debug, clap::Args)]
pub struct DeleteArgs {
    /// History file path
    pub path: PathBuf,

    /// Entry to delete.
    pub key: String,

    /// Throw an error if the target file does not exist, instead of createing a new file.
    #[arg(short = 'e', long = "strict", value_name = "NUM")]
    pub strict: bool,
}

pub fn parse_args() -> Command {
    let cli = Cli::parse();
    cli.cmd
}
