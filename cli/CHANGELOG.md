# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.1](https://github.com/jdrouet/pcloud/compare/pcloud-cli-v1.2.0...pcloud-cli-v1.2.1) - 2025-05-07

### Other

- updated the following local packages: pcloud

## [1.2.0](https://github.com/jdrouet/pcloud/compare/pcloud-cli-v1.1.0...pcloud-cli-v1.2.0) - 2025-05-03

### Added

- rewrite to make client simpler to use ([#105](https://github.com/jdrouet/pcloud/pull/105))

### Other

- *(deps)* bump all outdated deps
- *(deps)* Bump tokio from 1.39.2 to 1.43.1 ([#106](https://github.com/jdrouet/pcloud/pull/106))
- *(cli)* rewrite to simplify command ([#96](https://github.com/jdrouet/pcloud/pull/96))
- *(lib)* allow to stream uploads
- *(lib)* use serder to serialize params ([#94](https://github.com/jdrouet/pcloud/pull/94))
- remove unused dependencies

## [1.1.0](https://github.com/jdrouet/pcloud/compare/pcloud-cli-v1.0.0...pcloud-cli-v1.1.0) - 2024-08-11

### Added
- create multipart file upload command and use it in cli ([#62](https://github.com/jdrouet/pcloud/pull/62))
- *(cli)* create upload queue running on multiple threads ([#59](https://github.com/jdrouet/pcloud/pull/59))
- *(cli)* create download queue and assign multiple threads for downloading files ([#58](https://github.com/jdrouet/pcloud/pull/58))

### Other
- update existing code ([#89](https://github.com/jdrouet/pcloud/pull/89))
