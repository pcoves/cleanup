use crate::error::Result;
use aws_sdk_ec2::{
    model::{Filter, Volume},
    Client,
};
use futures::future::join_all;
use std::ops::Deref;

#[derive(Default)]
pub struct DescribeVolumes {
    pub snapshot_id: Option<String>,
    pub name: Option<String>,
}

struct Filters(Option<Vec<Filter>>);

impl Deref for Filters {
    type Target = Option<Vec<Filter>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DescribeVolumes> for Filters {
    fn from(describe_volumes: DescribeVolumes) -> Self {
        let mut filters = vec![Filter::builder()
            .set_name(Some("status".to_owned()))
            .set_values(Some(vec!["available".to_owned()]))
            .build()];

        if let Some(snapshot_id) = describe_volumes.snapshot_id {
            filters.push(
                Filter::builder()
                    .set_name(Some("snapshot-id".to_owned()))
                    .set_values(Some(vec![snapshot_id]))
                    .build(),
            )
        }

        if let Some(name) = describe_volumes.name {
            filters.push(
                Filter::builder()
                    .set_name(Some("tag:Name".to_owned()))
                    .set_values(Some(vec![name]))
                    .build(),
            )
        }

        Filters(Some(filters))
    }
}

pub async fn describe_volumes(
    client: &Client,
    describe_volumes: DescribeVolumes,
) -> Result<Volumes> {
    Ok(Volumes(
        client
            .describe_volumes()
            .set_filters(Filters::from(describe_volumes).0)
            .send()
            .await?
            .volumes
            .unwrap_or(vec![]),
    ))
}

pub async fn delete_volume(client: &Client, volume_id: String) -> Result<()> {
    client
        .delete_volume()
        .set_volume_id(Some(volume_id))
        .send()
        .await?;

    Ok(())
}

pub struct Volumes(Vec<Volume>);

impl Deref for Volumes {
    type Target = Vec<Volume>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Volumes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (mut id, mut name, mut total): (usize, usize, usize) = (0, 0, 0);

        for volume in self.iter() {
            let len = volume
                .volume_id
                .as_ref()
                .map(|volume_id| volume_id.len())
                .unwrap_or(0);

            if len > id {
                id = len;
            }

            if let Some(tags) = volume.tags() {
                if let Some(tag) = tags
                    .iter()
                    .find(|tag| tag.key() == Some(&"Name".to_owned()))
                {
                    let len = tag.value().map(|value| value.len()).unwrap_or(0);
                    if len > name {
                        name = len;
                    }
                }
            }

            total += volume.size().unwrap_or(0) as usize;
        }
        let size = total.to_string().len();

        writeln!(f, "| {:id$} | {:name$} | {:>size$} |", "ID", "Name", "Size")?;

        for volume in self.iter() {
            let volume_id = volume.volume_id.as_ref().unwrap();
            let volume_name = volume
                .tags()
                .unwrap_or(&[])
                .iter()
                .find(|tag| tag.key() == Some(&"Name".to_owned()))
                .map(|tag| tag.value().unwrap())
                .unwrap_or("");
            let volume_size = volume.size().unwrap_or(0);

            writeln!(
                f,
                "| {:id$} | {:name$} | {:>size$}Go |",
                volume_id, volume_name, volume_size
            )?;
        }

        let size = id + name + total.to_string().len();
        writeln!(f, "| Total {:>size$}Go |", total)
    }
}

impl Volumes {
    pub async fn delete(&self, client: &Client) -> Result<()> {
        join_all(
            self.iter()
                .map(|volume| delete_volume(&client, volume.volume_id.as_ref().unwrap().clone()))
                .collect::<Vec<_>>(),
        )
        .await;

        Ok(())
    }
}
