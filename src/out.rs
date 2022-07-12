use crate::aws::{image::Images, snapshot::Snapshots, volume::Volumes};
use aws_sdk_ec2::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub enum Out {
    Images(Images),
    Snapshots(Snapshots),
    Volumes(Volumes),
}

impl Out {
    pub fn read(path: PathBuf) -> Self {
        let file = std::fs::File::open(path).expect("Disc read failure");
        serde_json::from_reader(file).expect("Deserialization failure")
    }

    pub fn write(&self, path: PathBuf) {
        let serialized = serde_json::to_string(&self).expect("Serialization failure");
        std::fs::write(path, serialized).expect("Disc write failure");
    }

    pub async fn cleanup(&self, client: &Client) {
        match self {
            Self::Images(images) => images.cleanup(&client).await,
            Self::Snapshots(snapshots) => snapshots.cleanup(&client).await,
            Self::Volumes(volumes) => volumes.cleanup(&client).await,
        }
    }
}

impl std::fmt::Display for Out {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(&self).expect("Serialization failure")
        )
    }
}
