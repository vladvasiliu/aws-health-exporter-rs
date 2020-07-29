# AWS Health Exporter
[![CI](https://github.com/vladvasiliu/aws-health-exporter-rs/workflows/CI/badge.svg)](https://github.com/vladvasiliu/aws-health-exporter-rs/actions?query=workflow%3ACI)
[![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/vladvasiliu/aws-health-exporter-rs.svg?style=flat)][docker hub]
[![License](https://img.shields.io/github/license/vladvasiliu/aws-health-exporter-rs.svg?style=flat)](COPYING)


A Prometheus exporter for [AWS Health] events.

## Status
This is still a work in progress, although it is already being used in production.

It exports all events from AWS with event type "scheduled change" and "issue".

See the [Changelog](CHANGELOG.md) for changes between versions.

### Known limitations

Check the [issues][github issues] for open bugs / enhancements.

Most notable limitations are:
* Doesn't export affected resources and accounts ([#18](https://github.com/vladvasiliu/aws-health-exporter-rs/issues/18))
* Doesn't export process information on Windows (this is a limitation of the underlying
    [Prometheus library][prometheus-rust])
* Attempt to bind to a used socket when using TLS panics instead of quitting gracefully.
    This is a limitation of the underlying [http][warp] library.

## Requirements

* An AWS account,
* AWS Business support or better - [official docs](https://docs.aws.amazon.com/health/latest/ug/health-api.html).


## Usage

Pass the `-h` flag for a list of expected options.

This exporter uses AWS IAM for authentication to the AWS API. Please see [AWS Credentials](#aws-credentials) for details.

Example for exporting organization events by assuming the role *aws-health-exporter* in account *123456789012* and
listening on port 9679 and on all the machine's ips:

```
aws_health_exporter --role=arn:aws:iam::123456789012:role/aws-health-exporter --organization -l [::]:9679
```



### Using Docker

An image is available on [Docker Hub]. Its entry point is the executable, so it can be used just like the standalone
binary.

```
docker run --rm -d --name aws-health-exporter --net=host vladvasiliu/aws-health-exporter-rs:latest -l [::]:9679
```

For help, pass the `-h` flag:

```
docker run --rm --name aws-health-exporter vladvasiliu/aws-health-exporter-rs:latest -h
```

### Behaviour

The exporter exposes two endpoints:

* `/status` can be used for a health check
* `/metrics` to gather the actual statistics


The export should behave as follows:

* An unknown endpoint will return a `HTTP 404`.
* If the exporter itself is OK a call to `/metrics` will always return `HTTP 200` and some metrics.
  Deviation from this behaviour is considered a bug.
* Status of the AWS API call is reflected by `aws_health_events_success` metric.


## AWS credentials

The exporter uses [Rusoto] to interact with the AWS API.

Credentials can be provided as follows:

* Environment variables: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`
* AWS credentials file: `~/.aws/credentials`
* ECS IAM profile
* EC2 IAM profile

Please follow the [AWS credentials best practices].

As organization health events can only be queried from the main account, you are likely to have to use a different role
in that account for actually querying the API. The exporter can assume such a role with the `--role` flag.

As the organization account can easily do a lot of damage to connected accounts, please make sure that the assumed role
only has the absolute minimum permissions.

### Required permissions

Below is an example IAM policy of a role allowed to query the health API for the organization.
You may want to further restrict access by adding request conditions to the policy.

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "VisualEditor0",
            "Effect": "Allow",
            "Action": [
                "health:DescribeAffectedEntities",
                "health:DescribeEventDetails"
            ],
            "Resource": "arn:aws:health:*::event/*/*/*"
        },
        {
            "Sid": "VisualEditor1",
            "Effect": "Allow",
            "Action": [
                "organizations:ListAccounts",
                "health:DescribeEventDetailsForOrganization",
                "health:DescribeAffectedEntitiesForOrganization",
                "health:DescribeEventAggregates",
                "health:DescribeEventTypes",
                "health:DescribeAffectedAccountsForOrganization",
                "health:DescribeEventsForOrganization",
                "health:DescribeEvents",
                "health:DescribeEntityAggregates"
            ],
            "Resource": "*"
        }
    ]
}
```


### Assumed role example

As a full example, let's assume the following:

* The organization account is 111111111111;
* The exporter runs in account 222222222222;
* The exporter runs with the base role *monitoring.aws-health-exporter* from the account 222222222222, obtained, say,
    from the ECS role;
* The exporter should assume the role *aws-health-query* from the account 111111111111 to do the actual querying.

#### Base role

* Create the base role *monitoring.aws-health-exporter* and provide its credentials to the exporter via the usual
    means
* Create and attach the below policy to the base role so that it can assume the target role:

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Action": "sts:AssumeRole",
            "Resource": "arn:aws:iam::111111111111:role/aws-health-query",
            "Effect": "Allow"
        }
    ]
}
```


#### Target role

* Create the role *aws-health-query* with *Another AWS account* as trusted entity and 222222222222 as the trusted
    account id.
* Update the trust relationship to only allow the base role to assume this role

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::222222222222:role/monitoring.aws-health-exporter"
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
```

Attach the policy defined [above](#required-permissions) to the role.

For more information on cross-account access, you may want to check this
[AWS tutorial](https://docs.aws.amazon.com/IAM/latest/UserGuide/tutorial_cross-account-with-roles.html).

#### Run the exporter

```
aws_health_exporter --role=arn:aws:iam::111111111111:role/aws-health-query --organization
```


##  Building

This can be built on Linux, MacOS and Windows. As development happens on the latest stable versions of the Rust
toolchain and OS, there is no guarantee that older versions work.

```
cargo build --release
```

### Dependencies

Most notable dependencies:
* [prometheus-rust] for metrics exposure
* [Rusoto] for AWS API interaction
* [Tokio] for the async runtime
* [warp] for serving HTTP


## Contributing

Any contributions are welcome. Please open an issue or PR if you find any bugs or would like to propose an enhancement.


## Related projects

* [Azure Health Exporter](https://github.com/FXinnovation/azure-health-exporter)
* [AWS Health Exporter](https://github.com/Jimdo/aws-health-exporter)

The last one does pretty much the same thing as this exporter.
The reason I started a new project is two-fold:

* The above project doesn't look like it's maintained;
* I'm in the process of learning Rust and this looked like a good project.


## License

This project is released under the terms of the GNU General Public License, version 3.
Please see [`COPYING`](COPYING) for the full text of the license.


[aws api]: <https://docs.aws.amazon.com/health/latest/APIReference/API_DescribeEvents.html> "AWS API reference"
[aws credentials best practices]: <https://docs.aws.amazon.com/general/latest/gr/aws-access-keys-best-practices.html> "AWS access keys best practices"
[aws health]: <https://docs.aws.amazon.com/health/latest/ug/what-is-aws-health.html> "AWS Health"

[prometheus-rust]: <https://docs.rs/prometheus/> "Rust Prometheus documentation"
[rusoto]: <https://github.com/rusoto/rusoto> "Rust AWS SDK"
[tokio]: <https://tokio.rs/> "Tokio Homepage"
[warp]: <https://docs.rs/warp/> "Warp documentation"

[docker hub]: <https://hub.docker.com/r/vladvasiliu/aws-health-exporter-rs> "Docker Hub"
[github issues]: <https://github.com/vladvasiliu/aws-health-exporter-rs/issues> "GitHub Issues"
[roadmap]: <https://github.com/vladvasiliu/aws-health-exporter-rs/labels/enhancement> "Enhancements"
