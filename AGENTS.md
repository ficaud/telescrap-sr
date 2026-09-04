# AGENTS.md

This file is the instruction manual for AI coding agents (and new contributors)
working on this repository. It describes the project structure, the role of each
source file, the coding conventions, and the mandatory build/test/lint workflow.
Always read it before making changes; keep it up to date when the layout or the
conventions evolve.

## Project overview

`telescrap-sr` is a Rust workspace implementing a ticket-resale scraping bot for
the Stade Rochelais rugby club. It periodically scrapes the club's ticketing
website for resale seats, filters the results, and sends notifications via
Telegram. A web admin panel allows reconfiguring the scanner at runtime.

Key characteristics:

- Edition 2024; binary in `src/main.rs`, libraries in `crates/*`.
- Scrapes two clubs today: Stade Rochelais (implemented) and Union Bordeaux
  Bègles (not yet implemented).
- Supports passive mode (notify only) and aggressive mode (auto add-to-cart).
- Persists encounters in a `redb` embedded database; caches resale links between
  scans to avoid re-parsing the full match list.
- Runs on `tokio`; the scan loop and the admin panel communicate through
  `tokio::sync::watch` channels.

## Workspace layout

- Crates: `parser`, `scanner`, `telegram-notifier`, `admin-panel`, `filter`, `db-reader`
- Dependency order: `parser` -> `scanner` -> `telegram-notifier`; `filter` used by `scanner`; `admin-panel` standalone
- Crate internal structure: `lib.rs` exposing `interface` / `controller` / `app` / `core`

## Repository layout (role of each `.rs` file)

`src/main.rs` — binary entry point. Wires `TelegramNotifier`, `ScannerHandle`, `admin_panel` and the `watch` channels together, then waits for Ctrl+C.

### `crates/parser` (scraping: fetch HTML, parse matches/seats, persist)

- `src/lib.rs` — exposes `interface` / `controller` / `app` / `core`.
- `src/main.rs` — standalone binary that prints DB contents (`match_manager::print_db_contents`).
- `core/club.rs` — `Club` struct + `ClubType` enum (Stade Rochelais, UBB).
- `core/seat.rs` — `Seat`, `SeatInfo`, `SeatComposition`, `SeatAction` data structures.
- `core/encounter.rs` — `Encounter`, `MatchNature` and French-date parsing helpers.
- `controller/html_extract.rs` — `FetchHtml` trait (get HTML, add-to-cart).
- `controller/encounter_store.rs` — `EncounterRecord` + `StoreEncounters` trait (DB sync).
- `app/clubs/parsers.rs` — `ParseMatch`, `ParseSeat`, `ParseSeatPreview` traits.
- `app/clubs/larochelle/parse_match.rs` — Stade Rochelais match-list HTML parser.
- `app/clubs/larochelle/parse_seat.rs` — Stade Rochelais seat parser (Drupal JSON + context).
- `app/clubs/larochelle/parse_seat_preview.rs` — Pacifa3D seat preview image resolver.
- `interface/club_manager.rs` — maps `ClubType` -> `Club` (name + ticketing URL).
- `interface/match_manager.rs` — top-level parser API (seats from matches/title, add-to-cart, preview).
- `interface/curl/web.rs` — `WebClient` (`FetchHtml` impl), shop login + add-to-cart HTTP flows.
- `interface/curl/proxy/proxy_core.rs` — `ProxyManager` (pool, rotation, failover).
- `interface/curl/proxy/proxy_api.rs` — `ProxyMode`, global `PROXY_MANAGER`/`PROXY_ENABLED`, retry logic.
- `interface/storage/redb.rs` — `StorageError` type.
- `interface/storage/storage_match.rs` — `EncounterStore` (`StoreEncounters` via redb).
- `interface/storage/storage_state.rs` — `BotStateStore` (pinned Telegram message id).

### `crates/filter` (filtering rules applied to scan results)

- `src/lib.rs` — exposes `filter` module.
- `filter/filter.rs` — `Filter` trait + criteria accessors.
- `filter/filter_chain.rs` — `FilterChain` (AND of filters).
- `filter/aggressive_chain.rs` — `AggressiveChain` (chain + side-effect action).
- `filter/rule.rs` — `Rule` / `RuleSet` (OR of rules, each with an action).
- `filter/config/price.rs` — `PriceFilter`.
- `filter/config/seat.rs` — `SeatPositionFilter` (+ consecutive-seat grouping).
- `filter/config/encounter.rs` — `EncounterFilter` (title match).

### `crates/scanner` (periodic scan loop, diffing, notifications)

- `src/lib.rs` — exposes `interface` / `controller` / `core` / `app`.
- `core/scan.rs` — `ScanMode`, `ScanConfig`, `ScanResult`.
- `core/app_state.rs` — `AppState` (Running / Stopped).
- `core/config_file.rs` — `config_scan.json` load/write + `ScanConfig` serde conversion.
- `app/scan_task.rs` — `ScanTask`: the periodic loop, applies filters, notifies.
- `app/diff.rs` — `diff()` comparing two seat sets (`DiffType` / `DiffResult`).
- `controller/notify.rs` — `Notify` trait (send / send_photo / send_and_pin / edit_message).
- `interface/runner.rs` — `ScannerHandle` (configure + spawn/stop the scan task).
- `interface/notifiers/console.rs` — `ConsoleNotifier` (debug notifier).

### `crates/telegram-notifier`

- `src/lib.rs` — exports `TelegramNotifier`.
- `src/notifier.rs` — `TelegramNotifier` (`Notify` impl via teloxide), status message.
- `core/bot_state.rs` — `BotState` (pinned message id).

### `crates/admin-panel` (web UI to reconfigure the scanner at runtime)

- `src/lib.rs` — exposes `interface`.
- `src/main.rs` — standalone binary running `admin_panel::run`.
- `interface/admin_panel.rs` — axum server: `/` form, `/config` + `/state` endpoints.

### `crates/db-reader`

- `src/main.rs` — CLI tool that dumps the redb encounters table as a table.

## Build / Test / Lint

- `cargo build`, `cargo run`
- `cargo test --workspace --lib` (exact CI command)
- `cargo clippy` (devcontainer `rust-analyzer.check.command`)
- `cargo fmt`

### Testing policy

- **Every new development MUST be tested.** Run `cargo test --workspace --lib` before
  finishing any change; add or update inline unit tests (`#[cfg(test)] mod tests`)
  to cover new logic. Do not consider a task complete while tests fail.

## Code conventions

- English doc comments (`///` with `# Arguments` / `# Returns`); some user-facing error messages are in French
- Unit tests inline in `#[cfg(test)] mod tests`
- `SPDX-License-Identifier: GPL-3.0-or-later` headers in shell scripts

## Git & commits (human responsibility)

- NEVER commit, tag, or push. The human does all commits.
- Stop after editing files; leave validation and committing to the human.

## Secrets & ignored files (never commit)

- `.env`, `.private/`, `config_scan.json`, `proxies.txt`, `target/`, `docker-compose.yml`

## Devcontainer

- `.devcontainer/` (Dockerfile + entrypoint.sh); lazygit `overrideGpg: true` and `xclip` already provisioned

## Bot operating URLs

Ticketing sites the bot scrapes (mapped from `ClubType` in `crates/parser/src/interface/club_manager.rs`):

- Stade Rochelais: `https://billetterie.staderochelais.com/fr`
- Union Bordeaux Bègles: `https://billetterie.ubbrugby.com/fr`

Other endpoints:

- Login: `https://billetterie.staderochelais.com/fr/user/login`
- Basket: `https://billetterie.staderochelais.com/fr/basket`
- Seat preview (Pacifa3D): `https://static.pacifa3d.com/StadeMarcelDeflandreLAROCHELLE/WIMS/STADE-ROCHELAIS_RUGBY_v2025-2026/`
- Proxy list (proxyscrape API): `https://api.proxyscrape.com/v4/free-proxy-list/get?...`

## Useful docs

- `README.md`, `doc/*.md`, `CHANGELOG.md`, `DOCKER_DEPLOYMENT.md`, `ARCHITECTURE.md`, `DB_READER.md`, `ADMIN_PANEL.md`,  `CONTRIBUTING.md` (currently "TBD")
