use rusoto_core::RusotoError;
use rusoto_ec2::{
    DeleteSnapshotError, DeleteVolumeError, DeregisterImageError, DescribeImagesError,
    DescribeInstancesError, DescribeSnapshotsError, DescribeVolumesError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DescribeImages(#[from] RusotoError<DescribeImagesError>),

    #[error(transparent)]
    DeregisterImage(#[from] RusotoError<DeregisterImageError>),

    #[error(transparent)]
    DescribeInstances(#[from] RusotoError<DescribeInstancesError>),

    #[error(transparent)]
    DescribeSnapshots(#[from] RusotoError<DescribeSnapshotsError>),

    #[error(transparent)]
    DeleteSnapshot(#[from] RusotoError<DeleteSnapshotError>),

    #[error(transparent)]
    DescribeVolumes(#[from] RusotoError<DescribeVolumesError>),

    #[error(transparent)]
    DeleteVolume(#[from] RusotoError<DeleteVolumeError>),
}

pub type Result<T> = std::result::Result<T, Error>;
