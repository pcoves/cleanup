use crate::{
    aws::volume::{Builder as VolumesBuilder, DescribeVolumes, Volumes},
    error::Result,
};
use aws_sdk_ec2::{
    model::{Filter, Snapshot},
    output::DescribeSnapshotsOutput,
    Client,
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct DescribeSnapshots {
    pub name: Option<String>,
    pub snapshot_id: Option<String>,
}

impl DescribeSnapshots {
    pub fn name(name: Option<String>) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn snapshot_id(snapshot_id: Option<String>) -> Self {
        Self {
            snapshot_id,
            ..Default::default()
        }
    }
}

struct Filters(Option<Vec<Filter>>);

impl From<DescribeSnapshots> for Filters {
    fn from(describe_snapshots: DescribeSnapshots) -> Self {
        let mut filters = vec![];

        if let Some(name) = describe_snapshots.name {
            filters.push(
                Filter::builder()
                    .set_name(Some("tag:Name".to_owned()))
                    .set_values(Some(vec![name]))
                    .build(),
            );
        }

        if let Some(id) = describe_snapshots.snapshot_id {
            filters.push(
                Filter::builder()
                    .set_name(Some("snapshot-id".to_owned()))
                    .set_values(Some(vec![id]))
                    .build(),
            );
        }

        Filters(Some(filters))
    }
}

pub struct Builder<'a> {
    client: &'a Client,
    output: DescribeSnapshotsOutput,
}

impl<'a> Builder<'a> {
    pub async fn new(
        client: &'a Client,
        describe_snapshots: DescribeSnapshots,
    ) -> Result<Builder<'a>> {
        Ok(Self {
            client,
            output: client
                .describe_snapshots()
                .set_filters(Filters::from(describe_snapshots).0)
                .send()
                .await?,
        })
    }

    pub async fn build(self) -> Snapshots {
        Snapshots(if let Some(snapshots) = self.output.snapshots() {
            Some(
                join_all(
                    snapshots
                        .iter()
                        .map(|snapshot| Info::new(self.client, snapshot)),
                )
                .await,
            )
        } else {
            None
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Snapshots(Option<Vec<Info>>);

#[derive(Serialize, Deserialize)]
pub struct Info {
    id: String,
    name: String,
    size: i32,
    volumes: Option<Vec<Volumes>>,
}

impl Info {
    pub async fn new(client: &Client, snapshot: &Snapshot) -> Self {
        let mut acc: Vec<Volumes> = Vec::new();

        if let Ok(builder) = VolumesBuilder::new(
            &client,
            DescribeVolumes::snapshot_id(Some(
                snapshot
                    .snapshot_id()
                    .expect("Failed to read snapshot's ID")
                    .to_string(),
            )),
        )
        .await
        {
            acc.push(builder.build().await)
        }

        Self {
            id: snapshot
                .snapshot_id()
                .expect("Failed to read snapshot ID")
                .to_string(),
            name: snapshot
                .tags()
                .unwrap_or(&[])
                .iter()
                .find(|tag| tag.key() == Some(&"Name".to_owned()))
                .map(|tag| tag.value().unwrap())
                .unwrap_or("")
                .to_string(),
            size: snapshot
                .volume_size()
                .expect("Failed to read snapshot's size"),
            volumes: Some(acc),
        }
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        client
            .delete_snapshot()
            .snapshot_id(&self.id)
            .send()
            .await?;
        if let Some(volumes) = &self.volumes {
            join_all(volumes.iter().map(|volume| volume.cleanup(client))).await;
        }
        Ok(())
    }
}

impl Snapshots {
    pub async fn cleanup(&self, client: &Client) {
        if let Some(snapshots) = &self.0 {
            join_all(snapshots.iter().map(|volume| volume.delete(client))).await;
        }
    }
}

impl std::fmt::Display for Snapshots {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).expect("Serialization failure")
        )
    }
}
