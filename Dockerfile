FROM rust:1.41-buster as builder

WORKDIR /build
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN cargo install --path .

FROM debian:buster

LABEL description="Prometheus exporter for AWS Health"
LABEL maintainer="Vlad Vasiliu <vladvasiliun@yahoo.fr>"

ENV PORT=9679
EXPOSE $PORT

RUN apt-get update && apt-get install -y openssl curl && rm -rf /var/lib/apt
COPY --from=builder /usr/local/cargo/bin/aws_health_exporter /usr/local/bin/aws_health_exporter
HEALTHCHECK --interval=5s --timeout=3s CMD curl -sS http://127.0.0.1:$PORT/status -o /dev/null || exit 1
ENTRYPOINT ["aws_health_exporter"]
