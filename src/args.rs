use clap::{Parser, Subcommand};
use rusoto_core::Region;

/// Search for AMI's or orphan Snapshots to delete.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, default_value = "eu-west-1")]
    pub region: Region,

    /// Custom endpoint for testing purpose.
    #[clap(short, long)]
    pub endpoint: Option<String>,

    /// AMIs name (or prefix if ends with a *).
    #[clap(short, long)]
    pub name: Option<String>,

    /// Delete AMIs/Snapshots.
    #[clap(long)]
    pub apply: bool,

    /// If no command, handles orphan snapshots.
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// How many AMIs to keep.
    Keep(Keep),

    /// Keep AMIs based on their age.
    Before(Before),
}

#[derive(Debug, Parser)]
pub struct Keep {
    #[clap(default_value_t = 2)]
    pub keep: usize,
}

#[derive(Debug, Parser)]
pub struct Before {
    #[clap(short('H'), long, default_value_t = 0)]
    pub hours: i64,
    #[clap(short('D'), long, default_value_t = 0)]
    pub days: i64,
    #[clap(short('W'), long, default_value_t = 0)]
    pub weeks: i64,
}
