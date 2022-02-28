use crate::Result;
use rusoto_ec2::{filter, DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client};

pub async fn describe_instances(
    ec2_client: &Ec2Client,
    image_id: &str,
) -> Result<DescribeInstancesResult> {
    let describe_instance_request = DescribeInstancesRequest {
        filters: Some(vec![filter!("image-id", image_id.to_owned())]),
        ..Default::default()
    };

    Ok(ec2_client
        .describe_instances(describe_instance_request)
        .await?)
}
