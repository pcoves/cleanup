use crate::Result;
use rusoto_ec2::{
    DeleteSnapshotRequest, DescribeSnapshotsRequest, DescribeSnapshotsResult, Ec2, Ec2Client,
};

pub async fn decribe_snapshots(ec2_client: &Ec2Client) -> Result<DescribeSnapshotsResult> {
    Ok(ec2_client
        .describe_snapshots(DescribeSnapshotsRequest {
            owner_ids: Some(vec!["self".to_owned()]),
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
