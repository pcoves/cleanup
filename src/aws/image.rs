use crate::{
    aws::snapshot::{Builder as SnapshotsBuilder, DescribeSnapshots, Snapshots},
    error::Result,
};
use aws_sdk_ec2::{
    model::{Filter, Image},
    output::DescribeImagesOutput,
    Client,
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default)]
pub struct DescribeImages {
    pub names: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub snapshot_id: Option<String>,
}

struct Filters(Option<Vec<Filter>>);

impl From<DescribeImages> for Filters {
    fn from(describe_images: DescribeImages) -> Self {
        let mut filters = vec![];

        if describe_images.names.is_some() {
            filters.push(
                Filter::builder()
                    .set_name(Some("name".to_owned()))
                    .set_values(describe_images.names)
                    .build(),
            )
        }

        if let Some(tags) = describe_images.tags {
            let mut key_values = HashMap::new();

            for tag in tags {
                let mut splits = tag.split(" ");
                key_values
                    .entry(splits.next().expect("Failed to read tag's key").to_string())
                    .or_insert(vec![])
                    .append(&mut splits.map(|split| split.to_string()).collect::<Vec<_>>());
            }

            for (key, values) in key_values {
                filters.push(
                    Filter::builder()
                        .set_name(Some(format!("tag:{}", key)))
                        .set_values(Some(values))
                        .build(),
                )
            }
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

pub struct Builder<'a> {
    client: &'a Client,
    describe_images_output: DescribeImagesOutput,
}

impl<'a> Builder<'a> {
    pub async fn new(client: &'a Client, describe_images: DescribeImages) -> Result<Builder<'a>> {
        let describe_images_output = client
            .describe_images()
            .set_owners(Some(vec!["self".to_owned()]))
            .set_filters(Filters::from(describe_images).0)
            .send()
            .await?;

        let builder = Self {
            client,
            describe_images_output,
        }
        .sort();

        if let Some(images) = builder.describe_images_output.images() {
            log::info!("Found {} matching images", images.len());
        }

        let builder = builder.unused().await?;

        if let Some(images) = builder.describe_images_output.images() {
            log::info!("{} of them are unused", images.len());
        }

        Ok(builder)
    }

    pub fn exclude_names(self, names: Vec<String>) -> Self {
        let regex = regex::RegexSet::new(names).expect("Failed to build regex");

        let describe_images_output = DescribeImagesOutput::builder()
            .set_images(self.describe_images_output.images.map(|images| {
                images
                    .into_iter()
                    .filter(|image| {
                        regex
                            .matches(image.name().expect("Failed to read image name"))
                            .into_iter()
                            .collect::<Vec<_>>()
                            .is_empty()
                    })
                    .collect::<Vec<_>>()
            }))
            .build();

        if let Some(images) = describe_images_output.images() {
            log::info!("Kept {} after excluding by name", images.len());
        }

        Self {
            client: self.client,
            describe_images_output,
        }
    }

    pub fn exclude_tags(self, tags: Vec<String>) -> Self {
        let mut key_values = HashMap::new();

        for tag in tags {
            let mut splits = tag.split(" ");
            key_values
                .entry(splits.next().expect("Failed to read tag's key").to_string())
                .or_insert(vec![])
                .append(&mut splits.map(|split| split.to_string()).collect::<Vec<_>>());
        }

        let key_values = key_values
            .into_iter()
            .map(|(key, values)| {
                (
                    regex::Regex::new(&key).expect("Faild to compile regex"),
                    regex::RegexSet::new(values).expect("Failed to compile regex set"),
                )
            })
            .collect::<Vec<_>>();

        let describe_images_output = DescribeImagesOutput::builder()
            .set_images(self.describe_images_output.images.map(|images| {
                images
                    .into_iter()
                    .filter(|image| {
                        if let Some(tags) = image.tags() {
                            let mut matches = false;
                            for tag in tags {
                                if let (Some(key), Some(value)) = (tag.key(), tag.value()) {
                                    for (k, v) in &key_values {
                                        matches |= k.is_match(key) && v.is_match(value);
                                        if matches {
                                            break;
                                        }
                                    }
                                }
                            }
                            matches
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>()
            }))
            .build();

        if let Some(images) = describe_images_output.images() {
            log::info!("Kept {} after excluding by tag", images.len());
        }

        Self {
            client: self.client,
            describe_images_output,
        }
    }

    pub fn keep(self, keep: usize) -> Self {
        let output = DescribeImagesOutput::builder()
            .set_images(
                self.describe_images_output
                    .images
                    .map(|images| images.into_iter().skip(keep).collect::<Vec<_>>()),
            )
            .build();

        if let Some(images) = output.images() {
            log::info!("Will delete {} images and associated data", images.len());
        }

        Self {
            client: self.client,
            describe_images_output: output,
        }
    }

    pub fn before(self, before: DateTime<Utc>) -> Self {
        let output = DescribeImagesOutput::builder()
            .set_images(self.describe_images_output.images.map(|images| {
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
            describe_images_output: output,
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

    async fn unused(self) -> Result<Builder<'a>> {
        if let Some(images) = self.describe_images_output.images {
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

            Ok(Self {
                client: self.client,
                describe_images_output: DescribeImagesOutput::builder()
                    .set_images(Some(images))
                    .build(),
            })
        } else {
            Ok(self)
        }
    }

    fn sort(mut self) -> Self {
        if let Some(images) = &mut self.describe_images_output.images {
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

    pub async fn build(self) -> Result<Images> {
        Ok(Images(
            if let Some(images) = self.describe_images_output.images() {
                Some(join_all(images.iter().map(|image| Info::new(self.client, image))).await)
            } else {
                None
            },
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Images(Option<Vec<Info>>);

#[derive(Serialize, Deserialize)]
pub struct Info {
    id: String,
    name: String,
    creation_date: String,
    snapshots: Option<Vec<Snapshots>>,
}

impl Info {
    async fn new(client: &Client, image: &Image) -> Self {
        let mut acc: Vec<Snapshots> = Vec::new();

        if let Some(bdms) = image.block_device_mappings() {
            for bdm in bdms {
                if let Some(ebs) = bdm.ebs() {
                    if let Some(snapshot_id) = ebs.snapshot_id() {
                        if let Ok(builder) = SnapshotsBuilder::new(
                            &client,
                            DescribeSnapshots::snapshot_ids(Some(vec![snapshot_id.to_string()])),
                        )
                        .await
                        {
                            acc.push(builder.build().await)
                        };
                    }
                }
            }
        }

        Info {
            id: image
                .image_id()
                .expect("Failed to read image ID")
                .to_string(),
            name: image.name().expect("Failed to read image name").to_string(),
            creation_date: image
                .creation_date()
                .expect("Failed to read image's creation date")
                .to_string(),
            snapshots: Some(acc),
        }
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        client.deregister_image().image_id(&self.id).send().await?;
        if let Some(snapshots) = &self.snapshots {
            join_all(snapshots.iter().map(|snapshot| snapshot.cleanup(client))).await;
        }
        Ok(())
    }
}

impl Images {
    pub async fn cleanup(&self, client: &Client) {
        if let Some(snapshots) = &self.0 {
            join_all(snapshots.iter().map(|snapshot| snapshot.delete(client))).await;
        }
    }
}

impl std::fmt::Display for Images {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).expect("Serialization failure")
        )
    }
}
