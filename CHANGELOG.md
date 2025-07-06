# Changelog

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

