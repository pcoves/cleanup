use clap::Parser;
use cleanup::{args::Args, error::Result};
use rusoto_ec2::Ec2Client;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    let ec2_client = Ec2Client::new(args.region.clone());

    cleanup::cleanup(&ec2_client, args).await
}
