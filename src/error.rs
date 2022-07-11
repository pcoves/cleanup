use aws_sdk_ec2::{
    error::DescribeInstancesError,
    error::{DeleteSnapshotError, DescribeSnapshotsError},
    error::{DeleteVolumeError, DescribeVolumesError},
    error::{DeregisterImageError, DescribeImagesError},
    types::SdkError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DescribeInstances(#[from] SdkError<DescribeInstancesError>),

    #[error(transparent)]
    DescribeImages(#[from] SdkError<DescribeImagesError>),

    #[error(transparent)]
    DeregisterImage(#[from] SdkError<DeregisterImageError>),

    #[error(transparent)]
    DescribeSnapshots(#[from] SdkError<DescribeSnapshotsError>),

    #[error(transparent)]
    DeleteSnapshot(#[from] SdkError<DeleteSnapshotError>),

    #[error(transparent)]
    DescribeVolumes(#[from] SdkError<DescribeVolumesError>),

    #[error(transparent)]
    DeleteVolume(#[from] SdkError<DeleteVolumeError>),
}

pub type Result<T> = std::result::Result<T, Error>;
