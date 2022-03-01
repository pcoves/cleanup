use crate::Result;
use chrono::{DateTime, Utc};
pub use rusoto_ec2::Image;
use rusoto_ec2::{
    filter, DeregisterImageRequest, DescribeImagesRequest, DescribeImagesResult, Ec2, Ec2Client,
    Filter,
};

pub enum DescribeImage {
    Name(Option<String>),
    Snapshot(String),
}

impl DescribeImage {
    pub fn filters(&self) -> Vec<Filter> {
        match self {
            Self::Name(Some(name)) => vec![filter!("name", name)],
            Self::Name(None) => vec![],
            Self::Snapshot(snapshot_id) => {
                vec![filter!("block-device-mapping.snapshot-id", snapshot_id)]
            }
        }
    }
}

pub async fn describe_images(
    ec2_client: &Ec2Client,
    describe_image: DescribeImage,
) -> Result<DescribeImagesResult> {
    let describe_image_request = DescribeImagesRequest {
        owners: Some(vec!["self".to_owned()]),
        filters: Some(describe_image.filters()),
        ..Default::default()
    };

    Ok(ec2_client.describe_images(describe_image_request).await?)
}

pub fn sort_images_by_creation_date(images: &mut Vec<Image>) {
    images.sort_by(|lhs, rhs| {
        lhs.creation_date
            .as_ref()
            .map(|d| d.parse::<DateTime<Utc>>().unwrap())
            .unwrap()
            .cmp(
                &rhs.creation_date
                    .as_ref()
                    .map(|d| d.parse::<DateTime<Utc>>().unwrap())
                    .unwrap(),
            )
    });
}

pub fn filter_images_by_date(mut images: Vec<Image>, date: DateTime<Utc>) -> Vec<Image> {
    sort_images_by_creation_date(&mut images);

    images
        .into_iter()
        .filter(|image| {
            image
                .creation_date
                .as_ref()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap()
                < date
        })
        .collect::<Vec<_>>()
}

pub async fn deregister_image(ec2_client: &Ec2Client, image_id: String) -> Result<()> {
    Ok(ec2_client
        .deregister_image(DeregisterImageRequest {
            image_id,
            ..Default::default()
        })
        .await?)
}

pub fn pretty_print_images(images: &[Image]) {
    let (mut name, mut date) = (0, 0);

    for image in images.iter() {
        let len = image.name.as_ref().unwrap().len();
        if len > name {
            name = len;
        }

        let len = image.creation_date.as_ref().unwrap().len();
        if len > date {
            date = len;
        }
    }

    println!("| {:name$} | {:date$} |", "Name", "Date");
    for image in images.iter().rev() {
        println!(
            "| {:name$} | {:date$} |",
            image.name.as_ref().unwrap(),
            image.creation_date.as_ref().unwrap()
        );
    }
}
