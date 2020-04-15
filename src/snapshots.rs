use rusoto_core::RusotoError;
use rusoto_ec2::{
    filter, DeleteSnapshotError, DeleteSnapshotRequest, DescribeSnapshotsError,
    DescribeSnapshotsRequest, DescribeSnapshotsResult, Ec2, Ec2Client, Filter, Snapshot,
};

use crate::images::describe_images;

pub async fn describe_snapshots(
    ec2_client: &Ec2Client,
    filters: Option<Vec<Filter>>,
) -> Result<DescribeSnapshotsResult, RusotoError<DescribeSnapshotsError>> {
    let describe_snapshots_request = DescribeSnapshotsRequest {
        dry_run: None,
        filters,
        max_results: None,
        next_token: None,
        owner_ids: Some(vec!["self".to_string()]),
        restorable_by_user_ids: None,
        snapshot_ids: None,
    };

    ec2_client
        .describe_snapshots(describe_snapshots_request)
        .await
}

pub async fn delete_snapshots(
    ec2_client: &Ec2Client,
    apply: bool,
) -> Result<i64, Box<dyn std::error::Error>> {
    let snapshots = describe_snapshots(&ec2_client, None).await?.snapshots;

    let mut volume = 0;

    if let Some(snapshots) = snapshots.as_ref() {
        for snapshot in snapshots.iter() {
            let images = describe_images(
                &ec2_client,
                Some(vec![filter!(
                    "block-device-mapping.snapshot-id",
                    snapshot.snapshot_id.as_ref().unwrap()
                )]),
            )
            .await?
            .images;

            if let Some(images) = images {
                if images.is_empty() {
                    if apply {
                        volume += delete_snapshot(&ec2_client, &snapshot).await?;
                    } else {
                        println!(
                            "Snapshot {} has no associated AMI and can be deleted",
                            snapshot.snapshot_id.as_ref().unwrap()
                        );
                    }
                }
            }
        }
    }

    Ok(volume)
}

async fn delete_snapshot(
    ec2_client: &Ec2Client,
    snapshot: &Snapshot,
) -> Result<i64, RusotoError<DeleteSnapshotError>> {
    let delete_snapshot_request = DeleteSnapshotRequest {
        dry_run: None,
        snapshot_id: snapshot.snapshot_id.as_ref().unwrap().to_string(),
    };

    ec2_client
        .delete_snapshot(delete_snapshot_request)
        .await
        .and_then(|_| Ok(snapshot.volume_size.unwrap()))
}
