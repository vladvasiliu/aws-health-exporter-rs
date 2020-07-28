FROM rust:1.45-buster as builder

WORKDIR /build
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN cargo install --path .

FROM debian:buster

LABEL description="Prometheus exporter for AWS Health"
LABEL maintainer="Vlad Vasiliu <vladvasiliun@yahoo.fr>"

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt
COPY --from=builder /usr/local/cargo/bin/aws_health_exporter /usr/local/bin/aws_health_exporter
ENTRYPOINT ["aws_health_exporter"]
