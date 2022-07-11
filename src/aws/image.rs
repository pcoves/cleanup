use crate::{
    aws::snapshot::{DescribeSnapshots, Snapshots},
    error::Result,
};
use aws_sdk_ec2::{
    model::{Filter, Image},
    output::DescribeImagesOutput,
    Client,
};
use chrono::{DateTime, Utc};
use futures::future::join_all;

#[derive(Default)]
pub struct DescribeImages {
    pub name: Option<String>,
    pub tag: Option<String>,
    pub snapshot_id: Option<String>,
}

struct Filters(Option<Vec<Filter>>);

impl From<DescribeImages> for Filters {
    fn from(describe_images: DescribeImages) -> Self {
        let mut filters = vec![];

        if let Some(name) = describe_images.name {
            filters.push(
                Filter::builder()
                    .set_name(Some("name".to_owned()))
                    .set_values(Some(vec![name]))
                    .build(),
            )
        }

        if let Some(tag) = describe_images.tag {
            filters.push(
                Filter::builder()
                    .set_name(Some("tag:Name".to_owned()))
                    .set_values(Some(vec![tag]))
                    .build(),
            )
        }

        if let Some(id) = describe_images.snapshot_id {
            filters.push(
                Filter::builder()
                    .set_name(Some("block-device-mapping.snapshot-id".to_owned()))
                    .set_values(Some(vec![id]))
                    .build(),
            )
        }

        Filters(if filters.is_empty() {
            None
        } else {
            Some(filters)
        })
    }
}

pub struct Images<'a> {
    client: &'a Client,
    output: DescribeImagesOutput,
}

impl<'a> Images<'a> {
    pub async fn new(client: &'a Client, describe_images: DescribeImages) -> Result<Images<'a>> {
        let images = Self {
            client,
            output: client
                .describe_images()
                .set_owners(Some(vec!["self".to_owned()]))
                .set_filters(Filters::from(describe_images).0)
                .send()
                .await?,
        }
        .sort();

        if let Some(images) = images.output.images() {
            log::info!("Found {} matching images", images.len());
        }

        Ok(images.unused().await?)
    }

    pub async fn deregister(&self) -> Result<()> {
        if let Some(images) = self.output.images() {
            join_all(images.iter().map(|image| self.deregister_image(image))).await;
        }
        Ok(())
    }

    pub fn keep(self, keep: usize) -> Self {
        let output = DescribeImagesOutput::builder()
            .set_images(
                self.output
                    .images
                    .map(|images| images.into_iter().skip(keep).collect::<Vec<_>>()),
            )
            .build();

        if let Some(images) = output.images() {
            log::info!("Will delete {} images and associated data", images.len());
        }

        Self {
            client: self.client,
            output,
        }
    }

    pub fn before(self, before: DateTime<Utc>) -> Self {
        let output = DescribeImagesOutput::builder()
            .set_images(self.output.images.map(|images| {
                images
                    .into_iter()
                    .filter(|image| {
                        image
                            .creation_date()
                            .unwrap()
                            .parse::<DateTime<Utc>>()
                            .unwrap()
                            < before
                    })
                    .collect::<Vec<_>>()
            }))
            .build();

        if let Some(images) = output.images() {
            log::info!("Kept {} images", images.len());
        }

        Self {
            client: self.client,
            output,
        }
    }

    async fn is_image_used(client: &'a Client, image: &Image) -> Result<bool> {
        if let Some(reservations) = client
            .describe_instances()
            .set_filters(Some(vec![Filter::builder()
                .set_name(Some(("image-id").to_owned()))
                .set_values(Some(vec![image.image_id().unwrap().to_string()]))
                .build()]))
            .send()
            .await?
            .reservations()
        {
            Ok(!reservations.is_empty())
        } else {
            Ok(false)
        }
    }

    async fn unused(self) -> Result<Images<'a>> {
        if let Some(images) = self.output.images {
            let status = join_all(
                images
                    .iter()
                    .map(|image| Self::is_image_used(self.client, image)),
            )
            .await;

            // TODO: this might be enhanced using filter_map.
            let image_status = std::iter::zip(images, status);

            let filtered = image_status
                .into_iter()
                .filter(|(_, status)| !*status.as_ref().unwrap())
                .collect::<Vec<_>>();

            let images = filtered
                .into_iter()
                .map(|(image, _)| image)
                .collect::<Vec<_>>();

            log::info!("Found {} unused images", images.len());

            Ok(Self {
                client: self.client,
                output: DescribeImagesOutput::builder()
                    .set_images(Some(images))
                    .build(),
            })
        } else {
            Ok(self)
        }
    }

    fn sort(mut self) -> Self {
        if let Some(images) = &mut self.output.images {
            images.sort_by(|lhs, rhs| {
                lhs.creation_date()
                    .map(|d| d.parse::<DateTime<Utc>>().unwrap())
                    .unwrap()
                    .cmp(
                        &rhs.creation_date()
                            .map(|d| d.parse::<DateTime<Utc>>().unwrap())
                            .unwrap(),
                    )
                    .reverse()
            });
        }

        self
    }

    async fn deregister_image(&self, image: &Image) -> Result<()> {
        self.client
            .deregister_image()
            .image_id(image.image_id().unwrap())
            .send()
            .await?;

        if let Some(bdms) = image.block_device_mappings() {
            for bdm in bdms {
                if let Some(ebd) = bdm.ebs() {
                    if let Some(snapshot_id) = ebd.snapshot_id() {
                        if let Ok(snapshots) = Snapshots::new(
                            &self.client,
                            DescribeSnapshots::snapshot_id(Some(snapshot_id.to_string())),
                        )
                        .await
                        {
                            snapshots.delete().await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> std::fmt::Display for Images<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(images) = self.output.images() {
            let (mut id, mut name, mut date) = (0, 0, 0);

            for image in images {
                let len = image.image_id().map(|id| id.len()).unwrap_or(0);
                if len > id {
                    id = len;
                }

                let len = image.name().map(|name| name.len()).unwrap_or(0);
                if len > name {
                    name = len;
                }

                let len = image.creation_date().map(|date| date.len()).unwrap_or(0);
                if len > date {
                    date = len;
                }
            }
            let cardinal = images.len().to_string().len();

            writeln!(
                f,
                "| {:>cardinal$} | {:id$} | {:name$} | {:date$} |",
                "#", "ID", "Name", "Creation date"
            )?;

            for (index, image) in images.iter().enumerate() {
                writeln!(
                    f,
                    "| {:cardinal$} | {:id$} | {:name$} | {:date$} |",
                    index,
                    image.image_id().unwrap_or(""),
                    image.name().unwrap_or(""),
                    image.creation_date().unwrap_or("")
                )?;
            }

            Ok(())
        } else {
            writeln!(f, "No volumes found")
        }
    }
}
