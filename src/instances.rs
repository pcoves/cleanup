use rusoto_core::RusotoError;
use rusoto_ec2::{
    DescribeInstancesError, DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client,
    Filter,
};

pub async fn describe_instances(
    ec2_client: &Ec2Client,
    filters: Option<Vec<Filter>>,
) -> Result<DescribeInstancesResult, RusotoError<DescribeInstancesError>> {
    let describe_instances_request = DescribeInstancesRequest {
        dry_run: None,
        filters,
        instance_ids: None,
        max_results: None,
        next_token: None,
    };

    ec2_client
        .describe_instances(describe_instances_request)
        .await
}
