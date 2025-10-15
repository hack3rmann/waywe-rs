# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2019-12-01

* Sealed the `TypeUuidDynamic` trait so that it can't be implemented downstream. ([#5])
* Bump `syn` and `quote` dependencies to use version 1.0. ([#6])

[#5]: https://github.com/randomPoison/type-uuid/pull/5
[#6]: https://github.com/randomPoison/type-uuid/pull/6

## [0.1.1] - 2019-01-16

### Added

* UUIDs for primitives and some std types.
* More readible versions of the UUIDs for amethyst types.

## [0.1.0] - 2019-01-09

* `TypeUuid` and `TypeUuidDynamic` traits.
* `Bytes` type.
* Custom derive support for `TypeUuid`.
* UUIDs for some amethyst types.

[Unreleased]: https://github.com/randomPoison/type-uuid/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/randomPoison/type-uuid/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/randomPoison/type-uuid/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/randomPoison/type-uuid/compare/16d3369...v0.1.0
