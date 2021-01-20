FROM rust:1.49-alpine3.12 as builder

RUN apk add --no-cache musl-dev openssl-dev
WORKDIR /build
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN cargo install --path .

FROM alpine:3.13

LABEL description="Prometheus exporter for AWS Health"
LABEL maintainer="Vlad Vasiliu <vladvasiliun@yahoo.fr>"

RUN apk add --no-cache openssl ca-certificates
#RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt
COPY --from=builder /usr/local/cargo/bin/aws_health_exporter /usr/local/bin/aws_health_exporter
ENTRYPOINT ["aws_health_exporter"]
