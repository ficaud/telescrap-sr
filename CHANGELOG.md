# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.7] - 2026-05-24

### Added
- Release workflow that builds and publishes ARM64 binaries for `telescrap-sr` on GitHub Releases.

## [2.1.6] - 2026-05-24

### Added
- Docker Compose configuration for multi-instance deployment on a personal server (for example, a Raspberry Pi).
- Documentation for multi-instance deployment on a personal server (Raspberry Pi), including Docker Compose and manual setup instructions.
- Workflow dispatch inputs in CI to choose the runner for manual executions (GitHub-hosted or self-hosted).
- Example configuration for running two bot instances with Docker Compose (in the `examples/docker/` folder).

### Removed
- The old ci raspberry pi publishing workflow, replaced by a new workflow that builds and publishes the docker image to GitHub Container Registry.
- Telegram notification pinned message on startup, as it was not very useful.

## [2.1.5] - 2026-04-26

### Changed
- Improved diffference detection logic.
- Refactored the CI workflow to use a GitHub-hosted runner for builds and a self-hosted server runner for deployments across different environments (dev and prod).
- Translated one log from French to English in the `admin_panel` crate for consistency with the rest of the codebase.

## [2.1.4] - 2026-04-19

### Changed
- Improved configuration management by adding support for loading settings from a `config.json` file at startup and saving updates made from the admin web panel.

## [2.1.3] - 2026-04-18

### Fixed
- Fixed consecutive seat filtering: seats are now considered consecutive only when they are in the same row, including mixed even/odd seat numbers.

### Added
- Added a simple `db-reader` utility for debugging.
- Added an aggressive scan mode that logs in and adds a seat to the cart to confirm real resale availability and fetch more details.

### Changed
- Refactored filter configuration into a dedicated crate with a new `FilterConfig` struct for more flexible filter setup.
- Improved the admin panel structure and usability.
- Updated the deployment workflow to publish tagged versions as GitHub releases, and added documentation for multi-instance deployment on a personal server (for example, a Raspberry Pi).

## [2.1.2] - 2026-04-12

### Fixed
- Fix HTML tag identification for match parsing on the ticketing website

### Added
- Admin web panel (port 3000) to manage the bot configuration at runtime:
- Pinned message in the Telegram channel on startup showing the bot version, scan mode, interval and active filters

### Changed
- Scan configuration is now shared between the scanner and the admin panel via a `watch` channel, allowing live updates
- Deployment workflow now authenticates via SSH key instead of username/password

## [2.0.2] - 2026-04-10

### Added
- Seat preview image in Telegram notifications: when enabled, each seat with an available preview is sent as a photo message via the Pacifa3d 3D viewer API (configurable per scan filter).

### Changed
- Notifications are no longer sent when seats are removed from the resale platform, to avoid spamming the channel.

## [2.0.1] - 2026-04-06

### Added
- Better difference detection to identify new and removed seats more accurately, even when the seat numbers are not in the same order or when some seats are added or removed between scans.
- Filters : 
    - Filter by seat composition (e.g., only notify about seats that are together or only about single seats).
    - Filter by match type (e.g., only notify for certain matches titles).
    - Filter by price range (e.g., only notify about seats within a certain price range).
- Documentation for subcrates (still very light, but it's a start).
- Database integration to keep track of sent of each matches potential resale link and avoid scanning the main page for target's matches filtering.

### Changed
- Adding comments to the whole codebase to improve readability and maintainability, especially for the core logic of the parser and scanner.

## [2.0.0] - 2026-04-04

### Added
- Crate parser : responsible for parsing the HTML of the ticketing website and extract relevant information about the matches and the available seats.
- Crate scanner : responsible for performing the scans of the ticketing website, comparing the results with the previous scans, and notifying the Telegram bot of any changes.
- Crate telegram-notifier : responsible for sending notifications to the Telegram channel when resale tickets are detected;

### Changed
- The whole project's structure for better maintainability and readability (using library crates and clean architecture principles).

### Removed
- The old additional implementaion of the bot that made him more than a simple parser/notifier.
    - Message on telegram channel to notify users about upcoming matches at the beginning of the week, 1h before the match and at the kick-off.
    - log printed and registered in a local file
    - Administrator configuration on a private channel to start/stop get the status or setting the polling interval
    - Auomatic deletion of resale messages at the end of the week or day with a local database to keep track of sent messages

## [1.1.0] - 2026-03-30

### Added
- Message on telegram channel to notify users about upcoming matches at the beginning of the week, 1h before the match and at the kick-off.
- log printed and registered in a local file
- Administrator configuration on a private channel to start/stop get the status or setting the polling interval
- Auomatic deletion of resale messages at the end of the week or day with a local database to keep track of sent messages

### Changed
- Better looking notifications on telegram channel with HTML formatting
- Improve project's structure for better maintainability and readability
- update roadmap in the README, add tags, and add more details about the project in the README

## [1.0.0] - 2026-03-16

### Added

- Telegram bot that perform basic scans the Stade Rochelais ticketing website for resale tickets.
- Notifications sent to a Telegram channel when resale tickets are detected.
- Configuration options for bot token, chat IDs, and admin chat IDs.
- Telegram commands to allow supervision, configuration, start and stop of the bot.