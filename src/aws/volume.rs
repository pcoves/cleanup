use crate::error::Result;
use aws_sdk_ec2::{
    model::{Filter, Volume},
    output::DescribeVolumesOutput,
    Client,
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct DescribeVolumes {
    names: Option<Vec<String>>,
    snapshot_ids: Option<Vec<String>>,
}

impl DescribeVolumes {
    pub fn names(names: Option<Vec<String>>) -> Self {
        Self {
            names,
            ..Default::default()
        }
    }

    pub fn snapshot_ids(snapshot_ids: Option<Vec<String>>) -> Self {
        Self {
            snapshot_ids,
            ..Default::default()
        }
    }
}

struct Filters(Option<Vec<Filter>>);

impl From<DescribeVolumes> for Filters {
    fn from(describe_volumes: DescribeVolumes) -> Self {
        let mut filters = vec![Filter::builder()
            .set_name(Some("status".to_owned()))
            .set_values(Some(vec!["available".to_owned()]))
            .build()];

        if describe_volumes.snapshot_ids.is_some() {
            filters.push(
                Filter::builder()
                    .set_name(Some("snapshot-id".to_owned()))
                    .set_values(describe_volumes.snapshot_ids)
                    .build(),
            )
        }

        if describe_volumes.names.is_some() {
            filters.push(
                Filter::builder()
                    .set_name(Some("tag:Name".to_owned()))
                    .set_values(describe_volumes.names)
                    .build(),
            )
        }

        Filters(Some(filters))
    }
}

pub struct Builder<'a> {
    client: &'a Client,
    output: DescribeVolumesOutput,
}

impl<'a> Builder<'a> {
    pub async fn new(client: &'a Client, describe_volumes: DescribeVolumes) -> Result<Builder<'a>> {
        Ok(Self {
            client,
            output: client
                .describe_volumes()
                .set_filters(Filters::from(describe_volumes).0)
                .send()
                .await?,
        })
    }

    pub async fn build(self) -> Volumes {
        Volumes(if let Some(volumes) = self.output.volumes() {
            Some(join_all(volumes.iter().map(|volume| Info::new(self.client, volume))).await)
        } else {
            None
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    id: String,
    name: String,
    size: i32,
}

impl Info {
    pub async fn new(_client: &Client, volume: &Volume) -> Self {
        Self {
            id: volume
                .volume_id()
                .expect("Failed to read volume ID")
                .to_string(),
            name: volume
                .tags()
                .unwrap_or(&[])
                .iter()
                .find(|tag| tag.key() == Some(&"Name".to_owned()))
                .map(|tag| tag.value().unwrap())
                .unwrap_or("")
                .to_string(),
            size: volume.size().expect("Failed to read volume's size"),
        }
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        Ok(client
            .delete_volume()
            .volume_id(&self.id)
            .send()
            .await
            .and_then(|_| Ok(()))?)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Volumes(Option<Vec<Info>>);

impl Volumes {
    pub async fn cleanup(&self, client: &Client) {
        if let Some(volumes) = &self.0 {
            join_all(volumes.iter().map(|volume| volume.delete(client))).await;
        }
    }
}

impl std::fmt::Display for Volumes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).expect("Serialization failure")
        )
    }
}
