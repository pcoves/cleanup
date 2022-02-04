use rusoto_core::RusotoError;
use rusoto_ec2::{
    filter, DescribeImagesError, DescribeImagesRequest, DescribeImagesResult, Ec2, Ec2Client,
    Filter, Image,
};

use crate::instances::describe_instances;

pub async fn describe_images(
    ec2_client: &Ec2Client,
    filters: Option<Vec<Filter>>,
) -> Result<DescribeImagesResult, RusotoError<DescribeImagesError>> {
    let describe_images_request = DescribeImagesRequest {
        dry_run: None,
        executable_users: None,
        filters,
        image_ids: None,
        owners: Some(vec!["self".to_string()]),
        include_deprecated: None,
    };

    ec2_client.describe_images(describe_images_request).await
}

pub async fn delete_images(ec2_client: &Ec2Client) -> Result<i64, Box<dyn std::error::Error>> {
    let images = describe_images(&ec2_client, Some(vec![filter!("is-public", false)]))
        .await?
        .images;

    if let Some(images) = images.as_ref() {
        for image in images.iter() {
            let reservations = describe_instances(
                &ec2_client,
                Some(vec![filter!("image-id", image.image_id.as_ref().unwrap())]),
            )
            .await?
            .reservations;

            if let Some(reservations) = reservations {
                if reservations.is_empty() {
                    println!(
                        "AMI {} has no reservation and can be deleted",
                        image.image_id.as_ref().unwrap()
                    );
                }
            }
        }
    }

    Ok(0)
}

pub fn sort_images(images: &mut [Image]) {
    images.sort_by(|lhs, rhs| {
        if lhs.creation_date.as_ref().unwrap() < rhs.creation_date.as_ref().unwrap() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
}
