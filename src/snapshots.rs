use rusoto_core::RusotoError;
use rusoto_ec2::{
    filter, DeleteSnapshotError, DeleteSnapshotRequest, DescribeSnapshotsError,
    DescribeSnapshotsRequest, DescribeSnapshotsResult, Ec2, Ec2Client, Filter, Snapshot,
};
use std::collections::HashMap;

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

#[derive(Default)]
pub struct State {
    pub success: u64,
    pub failure: u64,
    pub volume: i64,
}

pub async fn delete_snapshots(
    ec2_client: &Ec2Client,
    apply: bool,
    keep: usize,
) -> Result<State, Box<dyn std::error::Error>> {
    let mut state = State::default();
    let mut hash_map = HashMap::new();

    let snapshots = describe_snapshots(&ec2_client, None).await?.snapshots;
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
                    let snapshots = hash_map
                        .entry(snapshot.volume_id.as_ref().unwrap())
                        .or_insert(vec![]);
                    snapshots.append(&mut vec![snapshot]);
                }
            }
        }
    }

    for (volume_id, snapshots) in hash_map.iter_mut() {
        if snapshots.len() < keep {
            continue;
        }

        println!("Volume id : {}", volume_id);

        snapshots.sort_by(|lhs, rhs| lhs.start_time.as_ref().cmp(&rhs.start_time.as_ref()));

        for snapshot in snapshots.iter().rev().skip(keep) {
            if apply {
                match delete_snapshot(&ec2_client, &snapshot).await {
                    Ok(volume) => {
                        state.success += 1;
                        state.volume += volume
                    }
                    Err(_) => state.failure += 1,
                }
            } else {
                state.success += 1;
            }
        }
    }

    Ok(state)
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
