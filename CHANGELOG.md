# Changelog

All notable changes to this project will be documented in this file.

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
