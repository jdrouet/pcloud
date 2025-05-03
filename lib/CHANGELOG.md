# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0](https://github.com/jdrouet/pcloud/compare/pcloud-v2.0.0...pcloud-v3.0.0) - 2025-05-03

### Added

- rewrite to make client simpler to use ([#105](https://github.com/jdrouet/pcloud/pull/105))
- add tracing

### Fixed

- use  instead of
- *(lib)* only import when feature enabled

### Other

- *(deps)* bump all outdated deps
- *(deps)* Bump tokio from 1.39.2 to 1.43.1 ([#106](https://github.com/jdrouet/pcloud/pull/106))
- *(cli)* rewrite to simplify command ([#96](https://github.com/jdrouet/pcloud/pull/96))
- *(lib)* allow to stream uploads
- *(lib)* use serder to serialize params ([#94](https://github.com/jdrouet/pcloud/pull/94))

## [2.0.0](https://github.com/jdrouet/pcloud/compare/pcloud-v1.1.0...pcloud-v2.0.0) - 2024-08-11

### Added
- create multipart file upload command and use it in cli ([#62](https://github.com/jdrouet/pcloud/pull/62))

### Other
- update existing code ([#89](https://github.com/jdrouet/pcloud/pull/89))
- remove fuse project ([#70](https://github.com/jdrouet/pcloud/pull/70))
- *(lib)* create named_params method for file and folder identifier ([#52](https://github.com/jdrouet/pcloud/pull/52))
- *(lib)* make all http commands be Send ([#57](https://github.com/jdrouet/pcloud/pull/57))
