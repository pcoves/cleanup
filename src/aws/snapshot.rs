use crate::{aws::volume::delete_volume, error::Result};
use aws_sdk_ec2::{
    model::{Filter, Snapshot},
    Client,
};
use futures::future::join_all;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;

#[derive(Default)]
pub struct DescribeSnapshots {
    pub name: Option<String>,
    pub snapshot_id: Option<String>,
}

struct Filters(Option<Vec<Filter>>);

impl Deref for Filters {
    type Target = Option<Vec<Filter>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

pub async fn describe_snapshots(
    client: &Client,
    describe_snapshots: DescribeSnapshots,
) -> Result<Snapshots> {
    Ok(Snapshots(
        client
            .describe_snapshots()
            .set_owner_ids(Some(vec!["self".to_owned()]))
            .set_filters(Filters::from(describe_snapshots).0)
            .send()
            .await?
            .snapshots
            .unwrap_or(vec![]),
    ))
}

pub async fn delete_snapshot(
    client: &Client,
    snapshot_id: String,
    volume_id: String,
) -> Result<()> {
    client
        .delete_snapshot()
        .set_snapshot_id(Some(snapshot_id))
        .send()
        .await?;

    delete_volume(&client, volume_id).await?;

    Ok(())
}

pub struct Snapshots(Vec<Snapshot>);

impl Deref for Snapshots {
    type Target = Vec<Snapshot>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Snapshots {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (mut id, mut name, mut total): (usize, usize, usize) = (0, 0, 0);

        for snapshot in self.iter() {
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

        for snapshot in self.iter() {
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
    }
}

impl Snapshots {
    pub async fn delete(&self, client: &Client) -> Result<()> {
        join_all(
            self.iter()
                .map(|snapshot| {
                    delete_snapshot(
                        &client,
                        snapshot.snapshot_id().map(|s| s.to_string()).unwrap(),
                        snapshot.volume_id().map(|s| s.to_string()).unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .await;

        Ok(())
    }
}
