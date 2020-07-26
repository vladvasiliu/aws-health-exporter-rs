ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

FROM ${BASE_IMAGE} AS builder

COPY . ./
RUN cargo build --release

FROM alpine:3.12

LABEL description="Prometheus exporter for AWS Health"
LABEL maintainer="Vlad Vasiliu <vladvasiliun@yahoo.fr>"

ARG PORT=9679
EXPOSE $PORT

RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/aws_health_exporter \
    /usr/local/bin/
HEALTHCHECK --interval=5s --timeout=3s CMD curl -sS http://127.0.0.1:$PORT/status -o /dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/aws_health_exporter"]

