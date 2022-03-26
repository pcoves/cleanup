use crate::error::Result;
use aws_sdk_ec2::{model::Filter, output::DescribeVolumesOutput, Client};
use futures::future::join_all;

#[derive(Default)]
pub struct DescribeVolumes {
    name: Option<String>,
    snapshot_id: Option<String>,
}

impl DescribeVolumes {
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

pub struct Volumes<'a> {
    client: &'a Client,
    output: DescribeVolumesOutput,
}

impl<'a> Volumes<'a> {
    pub async fn new(client: &'a Client, describe_volumes: DescribeVolumes) -> Result<Volumes<'a>> {
        Ok(Self {
            client,
            output: client
                .describe_volumes()
                .set_filters(Filters::from(describe_volumes).0)
                .send()
                .await?,
        })
    }

    pub async fn delete(&self) -> Result<()> {
        if let Some(volumes) = self.output.volumes() {
            join_all(volumes.iter().map(|volume| {
                self.client
                    .delete_volume()
                    .volume_id(volume.volume_id().unwrap())
                    .send()
            }))
            .await;
        }
        Ok(())
    }
}

impl<'a> std::fmt::Display for Volumes<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(volumes) = self.output.volumes() {
            let (mut id, mut name, mut total): (usize, usize, usize) = (0, 0, 0);

            for volume in volumes {
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

            for volume in volumes {
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
        } else {
            writeln!(f, "No volumes found")
        }
    }
}
