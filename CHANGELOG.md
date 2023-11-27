# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Fixed

- `with_global_lock` added to `Device::open`.
- Bumped MSRV to `1.58.0`.

## [0.0.2] - 2023-11-27

### Fixed

- Linux build failed due to conflicts between type definitions in the D3XX library.

## [0.0.1] - 2023-11-25

### Added

- Initial support for:
  - Device enumeration
  - Reading device configurations
  - Pipe I/O
  - GPIO control
  - Notifications
  - Overlapped (Asynchronous) I/O
- The rest of the crate
