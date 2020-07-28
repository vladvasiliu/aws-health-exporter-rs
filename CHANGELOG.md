# Changelog

This file documents notable changes for each version.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.3 - 2020-07-28

### Changed

* Stop retrying to get events after 10 attempts.


## 0.4.2 - 2020-07-26

### Changed

* Removed healthcheck from Docker as it didn't work well with configuring the listen socket.


## 0.4.1 - 2020-07-26

### Changed

* Updated dependencies
* Updated docker base image to Rust 1.45.


## 0.4.0 - 2020-07-26

### Added
* Support for AWS organizations.
* Retries with exponential backoff of describe_events calls.
