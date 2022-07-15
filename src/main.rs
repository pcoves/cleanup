mod aws;
mod error;
mod options;
mod out;

use clap::Parser;
use error::Result;
use options::{Command, Options, SubCommand};

use crate::{
    aws::{
        image::{Builder as ImagesBuilder, DescribeImages},
        snapshot::{Builder as SnapshotsBuilder, DescribeSnapshots},
        volume::{Builder as VolumesBuilder, DescribeVolumes},
    },
    out::Out,
};
use aws_sdk_ec2::Client;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let options: Options = Options::parse();
    log::info!("Options are: {options:?}");

    std::env::set_var("AWS_PROFILE", options.profile);
    std::env::set_var("AWS_REGION", options.region);

    let client = Client::new(&aws_config::load_from_env().await);

    match options.command {
        Command::Volume(command) => {
            if let Ok(builder) =
                VolumesBuilder::new(&client, DescribeVolumes::names(command.names)).await
            {
                let out = Out::Volumes(builder.build().await);
                if let Some(path) = command.output {
                    out.write(path);
                } else if command.apply {
                    out.cleanup(&client).await;
                } else {
                    println!("{out}");
                }
            }
        }
        Command::Snapshot(command) => {
            if let Ok(builder) =
                SnapshotsBuilder::new(&client, DescribeSnapshots::names(command.names)).await
            {
                let out = Out::Snapshots(builder.build().await);
                if let Some(path) = command.output {
                    out.write(path);
                } else if command.apply {
                    out.cleanup(&client).await;
                } else {
                    println!("{out}");
                }
            }
        }
        Command::Image(command) => {
            if let Ok(builder) = ImagesBuilder::new(
                &client,
                DescribeImages {
                    names: command.names,
                    tags: command.tags,
                    ..Default::default()
                },
            )
            .await
            {
                let builder = if let Some(names) = command.exclude_names {
                    builder.exclude_names(names)
                } else {
                    builder
                };

                let builder = if let Some(tags) = command.exclude_tags {
                    builder.exclude_tags(tags)
                } else {
                    builder
                };

                let builder = match command.subcommand {
                    SubCommand::Keep(keep) => builder.keep(keep.keep),
                    SubCommand::Before(before) => builder.before(before.into()),
                };

                let out = Out::Images(builder.build().await?);

                if let Some(path) = command.output {
                    out.write(path);
                } else if command.apply {
                    out.cleanup(&client).await;
                } else {
                    println!("{out}");
                }
            };
        }
        Command::Read(read) => {
            let out = Out::read(read.path);
            if read.apply {
                out.cleanup(&client).await;
            } else {
                println!("{out}")
            }
        }
    }

    Ok(())
}
