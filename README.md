# AWS Health Exporter
[![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/vladvasiliu/aws-health-exporter-rs.svg?style=flat)][docker hub]
[![License](https://img.shields.io/github/license/vladvasiliu/aws-health-exporter-rs.svg?style=flat)](COPYING)


This is a Prometheus exporter for AWS Health events.

## Status
It exports all events from AWS with event type "scheduled change" and "issue". While already used in production, it should not be considered battle tested.

### Known limitations

Check the [issues][github issues] for open bugs / enhancements.

Most notable limitations are:
* No organization support - *[Issue #14](https://github.com/vladvasiliu/aws-health-exporter-rs/issues/14)*
* Doesn't export process information on Windows (this is a limitation of the underlying [Prometheus library][prometheus-rust])
* Attempt to bind to a used socket when using TLS panics instead of quitting gracefully. This is a limitation of the underlying [http][warp] library.

## Usage

### Requirements

* An AWS account
* AWS Business support or better - [official docs](https://docs.aws.amazon.com/health/latest/ug/health-api.html)

### Using Docker

An image is available on [Docker Hub]. To run it:

```
docker run --rm -d --name aws-health-exporter vladvasiliu/aws-health-exporter-rs:latest
```

For help, pass the `-h` flag:

```
docker run --rm --name aws-health-exporter vladvasiliu/aws-health-exporter-rs:latest -h
```

###  Building

This is only tested with the latest stable version of Rust. To compile:

```
cargo build --release
```

#### Dependencies

Most notable dependencies:
* [prometheus-rust] for metrics exposure
* [Rusoto] for AWS API interaction
* [Tokio] for the async runtime
* [warp] for serving HTTP

### Running

Running with default configuration:

```
aws_health_exporter
```

The most useful argument is the listen socket:

```
aws_health_exporter -l 1.2.3.4:5678
```

Inspecting available options:

```
aws_health_exporter -h
```

#### Providing AWS credentials

The exporter uses [Rusoto] to interact with the AWS API.

Credentials can be provided as follows:

* Environment variables: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`
* AWS credentials file: `~/.aws/credentials`
* ECS IAM profile
* EC2 IAM profile

Please follow the [AWS credentials best practices].

### Scraping

The exporter exposes two endpoints:

* `/status` can be used for a health check
* `/metrics` to gather the actual statistics

#### Behaviour

* An unknown endpoint will return a `HTTP 404`.
* If the exporter itself is OK a call to `/metrics` will always return `HTTP 200` and some metrics.
  Deviation from this behaviour is considered a bug.
* Status of the AWS API call is reflected by `aws_health_events_success` metric.


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
[docker hub]: <https://hub.docker.com/repository/docker/vladvasiliu/aws-health-exporter-rs> "Docker Hub"
[github issues]: <https://github.com/vladvasiliu/aws-health-exporter-rs/issues> "GitHub Issues"
[prometheus-rust]: <https://docs.rs/prometheus/> "Rust Prometheus documentation"
[rusoto]: <https://github.com/rusoto/rusoto> "Rust AWS SDK"
[tokio]: <https://tokio.rs/> "Tokio Homepage"
[warp]: <https://docs.rs/warp/> "Warp documentation"
