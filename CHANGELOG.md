# Changelog

All notable changes to this project will be documented in this file.

## [0.3.4] - 2026-09-06

### Fixed

- Installer downloaded from the wrong repository and 404ed on every platform
- Installer now creates the install directory when it does not exist
- Installer now fails on unsupported Linux architectures instead of installing
  an x86-64 binary that cannot run
- Renamed release assets to match `uname -m`: `macos-x86` is now
  `macos-x86_64` and `linux-x86-64` is now `linux-x86_64`

## [0.3.3] - 2025-10-12

### Added

- Removed assets and templates directory from cookbook

### Fixed

- Fixed incorrect descriptions in README.tmpl.md
- Changed clap behavior to require subcommand
  - Previously this would assume the run subcommand if no subcommand was given
  - Currently shows a help message

## [0.3.2] - 2025-10-12

### Added

- First official release

### Fixed

- Changed unix step executor to bash
