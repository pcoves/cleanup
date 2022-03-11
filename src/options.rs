use clap::{Args, Parser, Subcommand};
use rusoto_core::Region;

/// Search for AMI's or orphan Snapshots to delete.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long, default_value = "eu-west-1")]
    pub region: Region,

    /// Custom endpoint for testing purpose.
    #[clap(short, long)]
    pub endpoint: Option<String>,

    /// Delete AMIs/Snapshots.
    #[clap(long)]
    pub apply: bool,

    /// If no command, handles orphan snapshots.
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search for orphaned volumes to delete.
    Volume(Volume),

    /// Search for orphaned snaphots to delete.
    Snapshot(Snapshot),

    /// Search for unused AMIs to delete.
    Ami(Ami),
}

#[derive(Debug, Args)]
pub struct Volume {
    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct Snapshot {
    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct Ami {
    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub tag: Option<String>,

    /// Filter by AMI name/prefix,
    #[clap(short, long)]
    pub name: Option<String>,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// How many AMIs to keep.
    Keep(Keep),

    /// AMI's expiration date.
    Before(Before),
}

#[derive(Debug, Parser)]
pub struct Keep {
    /// How many matching AMIs to keep.
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
