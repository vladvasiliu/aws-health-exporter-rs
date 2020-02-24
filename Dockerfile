FROM rust:1.41-buster as builder

ARG PORT=9679
#ARG VERSION

LABEL description="Prometheus exporter for AWS Health"
LABEL maintainer="Vlad Vasiliu <vladvasiliun@yahoo.fr>"

EXPOSE $PORT

WORKDIR /build
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
#RUN sed -i "s/^version.*/version = \"$VERSION\"/" Cargo.toml
RUN cargo install --path .

FROM debian:buster
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt
COPY --from=builder /usr/local/cargo/bin/aws_health_exporter /usr/local/bin/aws_health_exporter
HEALTHCHECK --interval=5s --timeout=1s --start-period=5s CMD curl -s http://127.0.0.1:$PORT -o /dev/null || exit 1
ENTRYPOINT ["aws_health_exporter"]
