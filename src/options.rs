use chrono::{DateTime, Duration, Utc};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Search for Image's or orphan Snapshots/Volume to delete.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long, default_value = "eu-west-1")]
    pub region: String,

    #[clap(short, long, default_value = "default")]
    pub profile: String,

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

    /// Search for unused images to delete.
    Image(Image),

    /// Read previously generated resource list to delete.
    Read(Read),
}

#[derive(Debug, Args)]
pub struct Volume {
    /// Effectively deletes volumes
    #[clap(long)]
    pub apply: bool,

    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub names: Option<Vec<String>>,

    /// Save result for later deletion
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct Snapshot {
    /// Effectively deletes snapshots
    #[clap(long)]
    pub apply: bool,

    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub names: Option<Vec<String>>,

    /// Save result for later deletion
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct Image {
    /// Effectively deletes images
    #[clap(long)]
    pub apply: bool,

    /// Filter by Tag:Name.
    #[clap(short, long)]
    pub tags: Option<Vec<String>>,

    /// Filter by image name/prefix,
    #[clap(short, long)]
    pub names: Option<Vec<String>>,

    /// Save result for later deletion
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// How many images to keep.
    Keep(Keep),

    /// Image's expiration date.
    Before(Before),
}

#[derive(Debug, Parser)]
pub struct Keep {
    /// How many matching images to keep.
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

impl Into<DateTime<Utc>> for Before {
    fn into(self) -> DateTime<Utc> {
        Utc::now()
            .checked_sub_signed(
                Duration::weeks(self.weeks)
                    + Duration::days(self.days)
                    + Duration::hours(self.hours),
            )
            .expect("Invalid date")
    }
}

#[derive(Debug, Args)]
pub struct Read {
    /// Effectively deletes images
    #[clap(long)]
    pub apply: bool,

    /// Path to read data from
    pub path: PathBuf,
}
