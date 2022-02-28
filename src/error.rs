use rusoto_core::RusotoError;
use rusoto_ec2::{
    DeleteSnapshotError, DeregisterImageError, DescribeImagesError, DescribeInstancesError,
    DescribeSnapshotsError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Images(#[from] RusotoError<DescribeImagesError>),

    #[error(transparent)]
    Deregister(#[from] RusotoError<DeregisterImageError>),

    #[error(transparent)]
    Instances(#[from] RusotoError<DescribeInstancesError>),

    #[error(transparent)]
    Snapshots(#[from] RusotoError<DescribeSnapshotsError>),

    #[error(transparent)]
    Delete(#[from] RusotoError<DeleteSnapshotError>),
}

pub type Result<T> = std::result::Result<T, Error>;
