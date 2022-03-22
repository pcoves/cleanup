use crate::{
    aws::snapshot::{describe_snapshots, DescribeSnapshots},
    error::Result,
};
use aws_sdk_ec2::{
    model::{Filter, Image},
    Client,
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use std::ops::Deref;

#[derive(Default)]
pub struct DescribeImages {
    pub name: Option<String>,
    pub tag: Option<String>,
    pub snapshot_id: Option<String>,
}

struct Filters(Option<Vec<Filter>>);

impl Deref for Filters {
    type Target = Option<Vec<Filter>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

pub struct Images(Vec<Image>);

impl Deref for Images {
    type Target = Vec<Image>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Images {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (mut id, mut name, mut date) = (0, 0, 0);

        for image in self.iter() {
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
        let cardinal = self.0.len().to_string().len();

        writeln!(
            f,
            "| {:>cardinal$} | {:id$} | {:name$} | {:date$} |",
            "#", "ID", "Name", "Creation date"
        )?;

        for (index, image) in self.iter().enumerate() {
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
    }
}

pub async fn describe_images(client: &Client, describe_images: DescribeImages) -> Result<Images> {
    Ok(Images(
        client
            .describe_images()
            .set_owners(Some(vec!["self".to_owned()]))
            .set_filters(Filters::from(describe_images).0)
            .send()
            .await?
            .images
            .unwrap_or(vec![]),
    )
    .sort())
}

pub async fn deregister_image(client: &Client, image_id: String) -> Result<()> {
    client
        .deregister_image()
        .set_image_id(Some(image_id))
        .send()
        .await?;

    Ok(())
}

impl Images {
    pub fn sort(mut self) -> Self {
        self.0.sort_by(|lhs, rhs| {
            lhs.creation_date()
                .map(|d| d.parse::<DateTime<Utc>>().unwrap())
                .unwrap()
                .cmp(
                    &rhs.creation_date()
                        .map(|d| d.parse::<DateTime<Utc>>().unwrap())
                        .unwrap(),
                )
        });

        Self(self.0)
    }

    pub fn filter(self, date: DateTime<Utc>) -> Self {
        Self(
            self.0
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
                .collect(),
        )
    }

    pub fn keep(self, keep: usize) -> Self {
        Self(self.0.into_iter().skip(keep).collect())
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        join_all(
            self.iter()
                .map(|image| {
                    deregister_image(&client, image.image_id().map(|s| s.to_owned()).unwrap())
                })
                .collect::<Vec<_>>(),
        )
        .await;

        for image in self.iter() {
            if let Some(ebms) = image.block_device_mappings() {
                for ebm in ebms {
                    if let Some(ebs) = ebm.ebs() {
                        describe_snapshots(
                            &client,
                            DescribeSnapshots {
                                snapshot_id: ebs.snapshot_id().map(|s| s.to_string()),
                                ..Default::default()
                            },
                        )
                        .await?
                        .delete(&client)
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
