use clap::Parser;
use cleanup::{args::Args, error::Result};
use rusoto_ec2::Ec2Client;
use rusoto_signature::Region;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    let region = if let Some(endpoint) = &args.endpoint {
        Region::Custom {
            name: args.region.name().to_string(),
            endpoint: endpoint.to_owned(),
        }
    } else {
        args.region.clone()
    };

    let ec2_client = Ec2Client::new(region);

    cleanup::cleanup(&ec2_client, args).await
}
