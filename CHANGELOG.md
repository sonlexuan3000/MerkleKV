# Changelog

All notable changes to the MerkleKV project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
