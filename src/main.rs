mod aws;
mod error;
mod options;

use clap::Parser;
use error::Result;
use options::{Command, Options, SubCommand};

use crate::aws::{
    image::{DescribeImages, Images},
    snapshot::{DescribeSnapshots, Snapshots},
    volume::{DescribeVolumes, Volumes},
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
            if let Ok(volumes) = Volumes::new(&client, DescribeVolumes::name(command.name)).await {
                if command.apply {
                    volumes.delete().await?;
                } else {
                    println!("{volumes}");
                }
            }
        }
        Command::Snapshot(command) => {
            if let Ok(snapshots) =
                Snapshots::new(&client, DescribeSnapshots::name(command.name)).await
            {
                if command.apply {
                    snapshots.delete().await?;
                } else {
                    println!("{snapshots}");
                }
            }
        }
        Command::Image(command) => {
            if let Ok(images) = Images::new(
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
                    SubCommand::Before(before) => images.before(before.into()),
                };

                if command.apply {
                    images.deregister().await?
                } else {
                    println!("{images}");
                }
            }
        }
    }

    Ok(())
}
