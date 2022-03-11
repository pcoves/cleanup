use crate::Result;
pub use rusoto_ec2::Snapshot;
use rusoto_ec2::{
    filter, DeleteSnapshotRequest, DescribeSnapshotsRequest, DescribeSnapshotsResult, Ec2,
    Ec2Client,
};

pub async fn describe_snapshots(
    ec2_client: &Ec2Client,
    tag: Option<String>,
) -> Result<DescribeSnapshotsResult> {
    Ok(ec2_client
        .describe_snapshots(DescribeSnapshotsRequest {
            owner_ids: Some(vec!["self".to_owned()]),
            filters: tag.map(|tag| vec![filter!("tag:Name", tag)]),
            ..Default::default()
        })
        .await?)
}

pub async fn delete_snapshot(ec2_client: &Ec2Client, snapshot_id: String) -> Result<()> {
    Ok(ec2_client
        .delete_snapshot(DeleteSnapshotRequest {
            snapshot_id,
            ..Default::default()
        })
        .await?)
}

pub fn pretty_print_snapshots(snapshots: &[Snapshot]) {
    if let Some(snapshot) = snapshots.get(0) {
        let (snap, vol, mut total) = (
            snapshot.snapshot_id.as_ref().unwrap().len(),
            snapshot.volume_id.as_ref().unwrap().len(),
            0,
        );

        for snapshot in snapshots.iter() {
            total += snapshot.volume_size.as_ref().unwrap();
        }
        let size = format!("{total}").len();

        println!(
            "| {:snap$} | {:vol$} | {:>size$} |",
            "Snapshot", "Volume", "Size"
        );
        for snapshot in snapshots.iter() {
            println!(
                "| {:snap$} | {:vol$} | {:>size$}Go |",
                snapshot.snapshot_id.as_ref().unwrap(),
                snapshot.volume_id.as_ref().unwrap(),
                snapshot.volume_size.as_ref().unwrap()
            );
        }

        println!(
            "| Total {:t$}   {:vol$} | {total}Go |",
            snapshots.len(),
            "",
            t = snap - 6
        );
    } else {
        println!("No such a thing as an orphaned snapshot.");
    }
}
