<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added
- Added a package dependency tree renderer

### Changed
- LuaPackages are now sourced from Roblox's CDN, rather than a local Studio installation
- LuaPackages are now discovered with a far more robust algorithm, allowing me to comfortably run the extractor automatically
- Script license headers are now fuzzily matched against a list of allowed licenses. A >= 95% match is required for a script to be considered licensed.
