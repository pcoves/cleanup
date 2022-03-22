mod aws;
mod error;
mod options;

use chrono::{Duration, Utc};
use clap::Parser;
use error::Result;
use options::{Command, Options, SubCommand};

use crate::aws::{
    image::{describe_images, DescribeImages},
    snapshot::{describe_snapshots, DescribeSnapshots},
    volume::{describe_volumes, DescribeVolumes},
};
use aws_sdk_ec2::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let options: Options = Options::parse();

    std::env::set_var("AWS_PROFILE", options.profile);
    std::env::set_var("AWS_REGION", options.region);

    let client = Client::new(&aws_config::load_from_env().await);

    match options.command {
        Command::Volume(command) => {
            if let Ok(volumes) = describe_volumes(
                &client,
                DescribeVolumes {
                    name: command.name,
                    ..Default::default()
                },
            )
            .await
            {
                if options.apply {
                    volumes.delete(&client).await?;
                } else {
                    println!("{volumes}");
                }
            }
        }
        Command::Snapshot(command) => {
            if let Ok(snapshots) = describe_snapshots(
                &client,
                DescribeSnapshots {
                    name: command.name,
                    ..Default::default()
                },
            )
            .await
            {
                if options.apply {
                    snapshots.delete(&client).await?;
                } else {
                    println!("{snapshots}");
                }
            }
        }
        Command::Image(command) => {
            if let Ok(images) = describe_images(
                &client,
                DescribeImages {
                    name: command.name,
                    tag: command.tag,
                    ..Default::default()
                },
            )
            .await
            {
                let images = match command.subcommand {
                    SubCommand::Keep(keep) => images.keep(keep.keep),
                    SubCommand::Before(before) => images.filter(
                        Utc::now()
                            .checked_sub_signed(
                                Duration::weeks(before.weeks)
                                    + Duration::days(before.days)
                                    + Duration::hours(before.hours),
                            )
                            .expect("Invalid date"),
                    ),
                };
                if options.apply {
                    images.delete(&client).await?;
                } else {
                    println!("{images}");
                }
            }
        }
    }

    Ok(())
}
