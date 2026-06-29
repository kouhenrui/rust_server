# Runtime Configuration (delta)

## Purpose

Define the environment variables that control runtime behavior of the service.

## MODIFIED Requirements

### Requirement: Primary prefix

Service-owned configuration variables MUST use the `THUMBOR_` prefix. The
logging subsystem additionally reads `RUST_LOG`.

#### Scenario: THUMBOR variables loaded

- GIVEN the service is starting
- WHEN it loads `Config`, cache config, or database config
- THEN it reads variables documented with the `THUMBOR_` prefix

### Requirement: Documented variables

Extended variable table — see canonical `openspec/specs/runtime-config/spec.md`
including `THUMBOR_LOG_LEVEL` and dotenv loading.

## ADDED Requirements

### Requirement: Dotenv loading

The binary MUST load `.env` via `dotenvy` before `logger::init()`.
