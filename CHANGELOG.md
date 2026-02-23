# Changelog

## 0.9.16 - 2026-02-23

### Removed

- Remove non-functional Letterboxd app, due to changes in Letterboxd site requirements

## 0.9.15 - 2026-02-23

### Fixed

- Update dependencies to fix bugs and vulnerabilities

## 0.9.14 - 2025-11-14

- **Main Server**: Remove compression for server-sent events for more immediate client-side processing

- **Letterboxd List App (frontend)**: Fix loading bar not appearing

## 0.9.13 - 2025-10-30

- **Main Server**: Actually add compression (got lost in a merge)

## 0.9.12 - 2025-10-30

- **Main Server**: Add stdout logging back in after improper merge


## 0.9.11 - 2025-10-29

- **Letterboxd List App (backend)**: Parallelize scraping for improved performance

- **General**: Revamp Ansible deployment for Kubernetes setup

- **Homepage**: Update main writeup, include mention of AI usage


## 0.9.10 - 2025-10-03

- **Main Server**: Add compression to images (really, anything over 2KiB)

## 0.9.9 - 2025-09-17

### Added

- **Main Server**: Add logging to stdout (finally)

## 0.9.8 - 2025-07-14

### Added

- **Homepage**: Add mention of Ansible deployment mechanism

## 0.9.7 - 2025-07-08

### Changed

- **Letterboxd List App (frontend)**: Changed built-in HTML warning when URL pattern is not matched


## 0.9.6 - 2025-07-08

### Fixed

- **Letterboxd List App (frontend)**: Fix film attribute dynamic list failure to reset after submission


## 0.9.5 - 2025-07-05

### Fixed

- **Letterboxd List App (backend)**: Fix snake-case-to-camel-case conversion on serialization for `ListInfo` type


## 0.9.4 - 2025-07-05

### Fixed

- **Letterboxd List App (frontend)**: Fix failed JSON parsing for Letterboxd list app rows received

- **Letterboxd List App (backend, Python)**: Fix connection refusal issue in Docker by binding to correct IP address (`0.0.0.0`)

## 0.9.3 - 2025-07-05

### Fixed

- Something important (can't recall)


## 0.9.2 - 2025-07-05

### Fixed

- **Guestbook (frontend)**: Fix double error message after overlong guestbook submission


## 0.9.1 - 2025-06-24

### Added

- **Main Server**: Add signal handler
- **Main Server**: Add compatibility for `camelCase` query variables


## 0.9.0 - 2025-06-24

### Added

- Add multi-stage Dockerfile build
- **Homepage**: Add banner
- **Homepage**: Add interactive React logo

### Changed

- **Letterboxd List App (frontend)**: Refactor to use React
- Change error logging to be more explicit
- **Letterboxd List App (backend, Rust)**: Change error event content to strings (instead of JSON)

