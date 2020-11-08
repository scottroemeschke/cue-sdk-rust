# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.0.1]

Initial release for CUE SDK version 3.0.55

## [v0.0.2]

### Changed
- Uses version `0.0.4` of the `cue-sdk-sys` crate, which has `Send` and `Sync` for various 
C structs coming from the iCUE SDK.

### Added
- Added `async` feature! Events and color buffer flushing can now be async/awaited.

## [v0.0.3]

### Changed
- Updates `cue_sdk_sys` to `0.0.5`.

### Fixed
- Fixed requiring all features of tokio, instead of just `sync`.
- Fixed async examples.
- Fixed various clippy warnings.
