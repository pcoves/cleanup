use cleanup::snapshots::delete_snapshots;

use dirs::home_dir;

use either::{Either, Left, Right};

use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_credential::{CredentialsError, ProfileProvider};
use rusoto_ec2::Ec2Client;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

use structopt::StructOpt;

use tini::Ini;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, help = "Apply command, defaults to false")]
    apply: bool,

    #[structopt(long, short, default_value = "default")]
    profile: String,

    #[structopt(short, long, default_value = "EuWest1")]
    region: Region,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let ec2_client = match provide_aws_credentials(&opt.profile)? {
        Left(s) => Ec2Client::new_with(HttpClient::new()?, s, opt.region),
        Right(p) => Ec2Client::new_with(HttpClient::new()?, p, opt.region),
    };

    println!(
        "{} GiB freed",
        delete_snapshots(&ec2_client, opt.apply).await?
    );

    Ok(())
}

fn provide_aws_credentials(
    profile: &str,
) -> Result<Either<StsAssumeRoleSessionCredentialsProvider, ProfileProvider>, CredentialsError> {
    let credentials = Ini::from_file(&home_dir().unwrap().join(".aws/credentials"))
        .expect("Could not read `~/.aws/credentials` file");

    if let Some(role_arn) = credentials.get(&profile, "role_arn") {
        Ok(Left(StsAssumeRoleSessionCredentialsProvider::new(
            StsClient::new(Region::EuWest1),
            role_arn,
            "Cleanup".to_string(),
            None,
            None,
            None,
            None,
        )))
    } else {
        let mut profile_provider = ProfileProvider::new()?;
        profile_provider.set_profile(profile);
        Ok(Right(profile_provider))
    }
}
