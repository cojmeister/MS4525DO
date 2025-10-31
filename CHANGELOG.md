# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-10-29

### Added
- Initial release of ms4525do driver
- Dual API support: blocking and async implementations
- Platform-agnostic design using `embedded-hal` and `embedded-hal-async`
- `no_std` support for embedded systems
- `std` support for desktop/server environments via feature flag
- Double-read validation for data freshness
- Built-in airspeed calculation from pressure and temperature
- Comprehensive error types with detailed descriptions
- Optional `defmt` logging support for embedded debugging
- Optional `log` logging support for flexible logging
- Zero dynamic allocation design
- Feature flags for flexible configuration
- CI/CD workflow with GitHub Actions
- Examples for both blocking and async usage
- Comprehensive documentation and README

### Features
- `async` (default): Enable async API with embassy-time
- `blocking`: Enable blocking/synchronous API
- `std`: Enable std support
- `defmt`: Enable defmt logging
- `log`: Enable log facade

[Unreleased]: https://github.com/cojmeister/ms4525do/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cojmeister/ms4525do/releases/tag/v0.1.0
