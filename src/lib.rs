pub mod args;
pub mod error;
mod images;
mod instances;
mod snapshots;

use crate::{
    error::Result,
    images::{
        deregister_image, describe_images, filter_images_by_date, pretty_print_images,
        sort_images_by_creation_date, DescribeImage, Image,
    },
    instances::describe_instances,
    snapshots::{decribe_snapshots, delete_snapshot, Snapshot},
};
use args::{Args, Before, Command, Keep};
use chrono::{Duration, Utc};
use rusoto_ec2::Ec2Client;
use std::collections::HashMap;

pub async fn cleanup(ec2_client: &Ec2Client, args: Args) -> Result<()> {
    if let Some(command) = args.command {
        // if let Some(mut images) = describe_images(&ec2_client, args.name.as_ref())
        if let Some(images) = describe_images(ec2_client, DescribeImage::Name(args.name))
            .await?
            .images
        {
            cleanup_images(ec2_client, images, command, args.apply).await?
        }
    } else if let Some(snapshots) = decribe_snapshots(ec2_client).await?.snapshots {
        cleanup_snapshots(ec2_client, snapshots, args.apply).await?;
    }

    Ok(())
}

async fn cleanup_images(
    ec2_client: &Ec2Client,
    mut images: Vec<Image>,
    command: Command,
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

    match command {
        Command::Keep(Keep { keep }) => {
            sort_images_by_creation_date(&mut images);
            images = images.into_iter().skip(keep).collect::<Vec<_>>();
        }
        Command::Before(Before { hours, days, weeks }) => {
            let date_threshold = Utc::now()
                .checked_sub_signed(
                    Duration::weeks(weeks) + Duration::days(days) + Duration::hours(hours),
                )
                .expect("Fail to compute date");

            if !apply {
                println!("Keeping AMIs younger than {}", date_threshold);
            }

            images = filter_images_by_date(images, date_threshold);
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
                delete_snapshot(ec2_client, snapshot_id).await?;
            }
        }
    } else {
        println!("Length: {}", images.len());
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
                    describe_images(ec2_client, DescribeImage::Snapshot(snapshot_id.clone()))
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
        }
    } else {
        println!("Orphans: {}", snapshots.len());
        for snapshot in snapshots {
            println!("{}", snapshot.snapshot_id.as_ref().unwrap());
        }
    }

    Ok(())
}
