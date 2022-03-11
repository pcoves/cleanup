pub mod error;
mod images;
mod instances;
pub mod options;
mod snapshots;
mod volumes;

use crate::{
    error::Result,
    images::{
        deregister_image, describe_images, filter_images_by_date, pretty_print_images,
        sort_images_by_creation_date, DescribeImage, Image,
    },
    instances::describe_instances,
    snapshots::{delete_snapshot, describe_snapshots, pretty_print_snapshots, Snapshot},
    volumes::{delete_volume, describe_volumes, pretty_print_volumes, DescribeVolumes, Volume},
};
use chrono::{Duration, Utc};
use options::{Before, Command, Keep, Options, SubCommand};
use rusoto_ec2::Ec2Client;
use std::collections::HashMap;

pub async fn cleanup(ec2_client: &Ec2Client, options: Options) -> Result<()> {
    match options.command {
        Command::Ami(ami) => {
            if let Some(images) = describe_images(
                ec2_client,
                DescribeImage {
                    name: ami.name,
                    tag: ami.tag,
                    ..Default::default()
                },
            )
            .await?
            .images
            {
                cleanup_images(ec2_client, images, ami.subcommand, options.apply).await?
            }
        }
        Command::Snapshot(snapshot) => {
            if let Some(snapshots) = describe_snapshots(ec2_client, snapshot.name)
                .await?
                .snapshots
            {
                cleanup_snapshots(ec2_client, snapshots, options.apply).await?;
            }
        }
        Command::Volume(volume) => {
            if let Some(volumes) = describe_volumes(
                ec2_client,
                DescribeVolumes {
                    name: volume.name,
                    ..Default::default()
                },
            )
            .await?
            .volumes
            {
                cleanup_volumes(ec2_client, volumes, options.apply).await?;
            }
        }
    }

    Ok(())
}

async fn cleanup_images(
    ec2_client: &Ec2Client,
    mut images: Vec<Image>,
    command: SubCommand,
    apply: bool,
) -> Result<()> {
    // Get all instances using the current image
    let image_instances = {
        let mut image_instances = HashMap::new();
        for image in &images {
            let image_id = image.image_id.as_ref().unwrap();
            image_instances
                .entry(image_id.clone())
                .or_insert_with(Vec::new)
                .push(describe_instances(ec2_client, image_id).await?);
        }
        image_instances
    };

    images = images
        .into_iter()
        .filter(|image| {
            let mut attached = false;
            for describe_instance_result in image_instances
                .get(image.image_id.as_ref().unwrap())
                .unwrap()
            {
                if let Some(reservations) = &describe_instance_result.reservations {
                    attached |= reservations.is_empty();
                }
            }
            attached
        })
        .collect::<Vec<_>>();

    images = match command {
        SubCommand::Keep(Keep { keep }) => {
            sort_images_by_creation_date(&mut images);
            images.into_iter().skip(keep).collect::<Vec<_>>()
        }
        SubCommand::Before(Before { hours, days, weeks }) => {
            let date_threshold = Utc::now()
                .checked_sub_signed(
                    Duration::weeks(weeks) + Duration::days(days) + Duration::hours(hours),
                )
                .expect("Fail to compute date");

            if !apply {
                println!("Keeping AMIs younger than {}", date_threshold);
            }

            filter_images_by_date(images, date_threshold)
        }
    };

    if apply {
        for image in &images {
            let snapshots_id = image
                .block_device_mappings
                .as_ref()
                .unwrap()
                .iter()
                .map(|block_device_mapping| {
                    block_device_mapping
                        .ebs
                        .as_ref()
                        .unwrap()
                        .snapshot_id
                        .as_ref()
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<_>>();

            deregister_image(ec2_client, image.image_id.as_ref().unwrap().clone()).await?;

            for snapshot_id in snapshots_id {
                delete_snapshot(ec2_client, snapshot_id.clone()).await?;

                if let Some(volumes) = describe_volumes(
                    ec2_client,
                    DescribeVolumes {
                        snapshot_id: Some(snapshot_id),
                        ..Default::default()
                    },
                )
                .await?
                .volumes
                {
                    for volume in volumes {
                        delete_volume(ec2_client, volume.volume_id.unwrap()).await?;
                    }
                };
            }
        }
    } else {
        println!("Matching images: {}", images.len());
        pretty_print_images(&images);
    }

    Ok(())
}

async fn cleanup_snapshots(
    ec2_client: &Ec2Client,
    mut snapshots: Vec<Snapshot>,
    apply: bool,
) -> Result<()> {
    let snapshot_images = {
        let mut snapshot_images = HashMap::new();
        for snapshot in &snapshots {
            let snapshot_id = snapshot.snapshot_id.as_ref().unwrap();
            snapshot_images
                .entry(snapshot_id.clone())
                .or_insert_with(Vec::new)
                .push(
                    describe_images(
                        ec2_client,
                        DescribeImage {
                            snapshot_id: Some(snapshot_id.to_string()),
                            ..Default::default()
                        },
                    )
                    .await?,
                );
        }
        snapshot_images
    };

    snapshots = snapshots
        .into_iter()
        .filter(|snapshot| {
            let mut attached = true;
            for describe_image in snapshot_images
                .get(snapshot.snapshot_id.as_ref().unwrap())
                .unwrap()
            {
                if let Some(images) = &describe_image.images {
                    attached &= images.is_empty();
                }
            }
            attached
        })
        .collect::<Vec<_>>();

    if apply {
        for snapshot in snapshots {
            delete_snapshot(ec2_client, snapshot.snapshot_id.unwrap()).await?;
            delete_volume(ec2_client, snapshot.volume_id.unwrap()).await?;
        }
    } else {
        pretty_print_snapshots(&snapshots);
    }

    Ok(())
}

async fn cleanup_volumes(
    ec2_client: &Ec2Client,
    mut volumes: Vec<Volume>,
    apply: bool,
) -> Result<()> {
    volumes = volumes
        .into_iter()
        .filter(|volume| volume.snapshot_id.is_some())
        .collect();

    if apply {
        for volume in volumes {
            delete_volume(ec2_client, volume.volume_id.unwrap()).await?;
        }
    } else {
        pretty_print_volumes(&volumes);
    }
    Ok(())
}
