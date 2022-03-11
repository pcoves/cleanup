use clap::Parser;
use cleanup::{error::Result, options::Options};
use rusoto_ec2::Ec2Client;
use rusoto_signature::Region;

#[tokio::main]
async fn main() -> Result<()> {
    let options: Options = Options::parse();

    let region = if let Some(endpoint) = &options.endpoint {
        Region::Custom {
            name: options.region.name().to_string(),
            endpoint: endpoint.to_owned(),
        }
    } else {
        options.region.clone()
    };

    let ec2_client = Ec2Client::new(region);

    cleanup::cleanup(&ec2_client, options).await
}
