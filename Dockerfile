ARG BASE_IMAGE=ekidd/rust-musl-builder:1.49.0
ARG RUN_IMAGE=alpine:3.13.0

FROM ${BASE_IMAGE} AS builder

ADD --chown=rust:rust . ./
RUN cargo build --release

FROM ${RUN_IMAGE}

ARG     VERSION
ARG     BUILD_DATE
ARG     GIT_HASH

LABEL org.opencontainers.image.version="$VERSION"
LABEL org.opencontainers.image.created="$BUILD_DATE"
LABEL org.opencontainers.image.revision="$GIT_HASH"
LABEL org.opencontainers.image.title="AWS Health Exporter"
LABEL org.opencontainers.image.description="Export AWS health events to Prometheus"
LABEL org.opencontainers.image.vendor="Vlad Vasiliu"
LABEL org.opencontainers.image.source="https://github.com/vladvasiliu/aws-health-exporter-rs"
LABEL org.opencontainers.image.authors="Vlad Vasiliu"
LABEL org.opencontainers.image.url="https://github.com/vladvasiliu/aws-health-exporter-rs"

RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/aws_health_exporter \
    /usr/local/bin/

ENTRYPOINT ["aws_health_exporter"]
