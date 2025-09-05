# Changelog

All notable changes to the MerkleKV project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.4.0] — 2025-09-05
### CI & Build
- Remove accidental C++ submodule in `clients/cpp/build/_deps/catch2-src`; preserve CMake FetchContent for Catch2.
- Add `.gitignore` rules to prevent build artifacts from being tracked.
- Align test discovery to include all 16/16 integration test files.
- Add nightly CI schedule to re-verify the matrix on `main` (02:17 UTC daily).
- Isolate optional integration tests with `@pytest.mark.slow` marker; run in dedicated job on schedule/manual dispatch.

### Code Hygiene
- Remove unused `info` import from `src/replication.rs`.

### Notes
- No protocol or public API changes. Minimal, surgical edits only.

### Added
- **Phase 1 Client Libraries**: Python (`merklekv`), Node.js (`@merklekv/client`), Go client libraries with full TCP protocol support
- **Phase 2 Client Libraries**: Java (`io.merklekv:client`), Rust (`merklekv-client`) with sync/async APIs
- **Phase 3 Client Libraries**: .NET (`MerkleKV.Client`), C++ (header-only), Ruby (`merklekv`), PHP (`merklekv/client`) completing 9-language support

### Fixed
- **Large Value Parser Corruption**: Server now handles values of arbitrary size using streaming line-based reading instead of fixed 1KB chunks
- **DELETE Response Semantics**: Server now returns `DELETED` for existing keys and `NOT_FOUND` for non-existing keys instead of always returning `OK`
- **Control Character Handling**: Server now allows tab (`\t`) characters in values while properly rejecting them in keys and commands. Note: Newline characters remain restricted due to text protocol design.
- **Port Configuration**: Standardized all clients to use default port 7379 matching server configuration

### Changed
- Updated documentation to reflect server-side fixes across README files
- Updated client library documentation to remove workarounds for fixed server issues

### Documentation
- Consolidated client library information from multiple phases
- Updated Known Issues sections to show resolved issues
- Improved protocol behavior documentation
- Added comprehensive client implementation summary

## Previous Releases

See git history for changes prior to this changelog.
