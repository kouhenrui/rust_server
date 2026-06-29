# Change: database-backend

## Why

The service requires a database connection at startup for future persistence
features, with swappable SQL and MongoDB backends.

## What Changes

- `DbProvider` trait and `DbClient` enum (postgres, mysql, sqlite, mongodb).
- `THUMBOR_DB_*` environment variables.
- Mandatory connect during `AppState::connect`.

## Capabilities

- `database-backend` — connection layer only (no ORM).

## Impact

- **Code:** `src/db/`, `src/state.rs`
- **Dependencies:** `sqlx`, `mongodb`

## Non-goals

- Migrations, schema management, query helpers
- Making database optional at startup

## Affected domains

- **database-backend** (created)
