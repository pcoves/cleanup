use crate::{
    aws::volume::{DescribeVolumes, Volumes},
    error::Result,
};
use aws_sdk_ec2::{model::Filter, output::DescribeSnapshotsOutput, Client};
use futures::future::join_all;

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

pub struct Snapshots<'a> {
    client: &'a Client,
    output: DescribeSnapshotsOutput,
}

impl<'a> Snapshots<'a> {
    pub async fn new(
        client: &'a Client,
        describe_snapshots: DescribeSnapshots,
    ) -> Result<Snapshots<'a>> {
        Ok(Self {
            client,
            output: client
                .describe_snapshots()
                .set_filters(Filters::from(describe_snapshots).0)
                .send()
                .await?,
        })
    }

    pub async fn delete(&self) -> Result<()> {
        if let Some(snapshots) = self.output.snapshots() {
            join_all(snapshots.iter().map(|snapshot| {
                self.client
                    .delete_snapshot()
                    .snapshot_id(snapshot.snapshot_id().unwrap())
                    .send()
            }))
            .await;

            join_all(
                join_all(snapshots.iter().map(|snapshot| {
                    Volumes::new(
                        self.client,
                        DescribeVolumes::snapshot_id(snapshot.snapshot_id().map(|s| s.to_string())),
                    )
                }))
                .await
                .into_iter()
                .map(|volumes| volumes.unwrap())
                .collect::<Vec<_>>()
                .iter()
                .map(|volumes| volumes.delete()),
            )
            .await;
        }
        Ok(())
    }
}

impl<'a> std::fmt::Display for Snapshots<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(snapshots) = self.output.snapshots() {
            let (mut id, mut name, mut total): (usize, usize, usize) = (0, 0, 0);

            for snapshot in snapshots {
                let len = snapshot.snapshot_id().map(|id| id.len()).unwrap_or(0);
                if len > id {
                    id = len;
                }

                if let Some(tags) = snapshot.tags() {
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

                total += snapshot.volume_size().unwrap() as usize;
            }
            let size = total.to_string().len();

            for snapshot in snapshots {
                let snapshot_id = snapshot.snapshot_id.as_ref().unwrap();
                let snapshot_name = snapshot
                    .tags()
                    .unwrap_or(&[])
                    .iter()
                    .find(|tag| tag.key() == Some(&"Name".to_owned()))
                    .map(|tag| tag.value().unwrap())
                    .unwrap_or("");
                let snapshot_size = snapshot.volume_size().unwrap_or(0);

                writeln!(
                    f,
                    "| {:id$} | {:name$} | {:>size$}Go |",
                    snapshot_id, snapshot_name, snapshot_size
                )?;
            }

            let size = id + name + total.to_string().len();
            writeln!(f, "| Total {:>size$}Go |", total)
        } else {
            writeln!(f, "No volumes found")
        }
    }
}
