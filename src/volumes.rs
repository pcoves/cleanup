use crate::Result;
pub use rusoto_ec2::Volume;
use rusoto_ec2::{
    filter, DeleteVolumeRequest, DescribeVolumesRequest, DescribeVolumesResult, Ec2, Ec2Client,
    Filter,
};

#[derive(Default)]
pub struct DescribeVolumes {
    pub snapshot_id: Option<String>,
    pub name: Option<String>,
}

impl DescribeVolumes {
    pub fn filters(self) -> Vec<Filter> {
        let mut filters = vec![filter!("status", "available")];

        if let Some(snapshot_id) = self.snapshot_id {
            filters.push(filter!("snapshot-id", snapshot_id));
        }

        if let Some(name) = self.name {
            filters.push(filter!("tag:Name", name));
        }

        filters
    }
}

pub async fn describe_volumes(
    ec2_client: &Ec2Client,
    describe_volumes: DescribeVolumes,
) -> Result<DescribeVolumesResult> {
    Ok(ec2_client
        .describe_volumes(DescribeVolumesRequest {
            filters: Some(describe_volumes.filters()),
            ..Default::default()
        })
        .await?)
}

pub async fn delete_volume(ec2_client: &Ec2Client, volume_id: String) -> Result<()> {
    Ok(ec2_client
        .delete_volume(DeleteVolumeRequest {
            volume_id,
            ..Default::default()
        })
        .await?)
}

pub fn pretty_print_volumes(volumes: &[Volume]) {
    let (mut id, mut total) = (0, 0);

    for volume in volumes.iter() {
        let len = volume.volume_id.as_ref().unwrap().len();
        if len > id {
            id = len;
        }

        total += volume.size.as_ref().unwrap();
    }

    let size = format!("{total}Go").len();

    println!("| {:id$} | {:>size$} |", "ID", "Size");
    for volume in volumes.iter() {
        println!(
            "| {:id$} | {:>size$} |",
            volume.volume_id.as_ref().unwrap(),
            volume
                .size
                .as_ref()
                .map(|size| format!("{size}Go"))
                .unwrap()
        );
    }
    println!("| Total {:t$} | {total}Go |", volumes.len(), t = id - 6);
}
