# Database Backend

## Purpose

Define the pluggable database connection layer required at application startup,
and its relationship to the `entity/` module for auth and Casbin tables.

## Requirements

### Requirement: Backend selection

The database backend MUST be selected by `THUMBOR_DB_BACKEND`:

| value | driver |
|---|---|
| `postgres` | sqlx `AnyPool` (PostgreSQL) |
| `mysql` | sqlx `AnyPool` (MySQL) |
| `sqlite` | sqlx `AnyPool` (SQLite) — **default** |
| `mongodb` | `mongodb::Client` |

#### Scenario: default sqlite

- GIVEN `THUMBOR_DB_BACKEND` is unset
- WHEN `DbBackendConfig::from_env()` runs
- THEN the backend is `sqlite`

#### Scenario: startup connects database

- GIVEN valid database configuration
- WHEN `AppState::connect` runs
- THEN a `database ready` log is emitted with the backend name

### Requirement: Connection configuration

The service MUST support `THUMBOR_DB_URL` or discrete fields:

| variable | purpose |
|---|---|
| `THUMBOR_DB_HOST` | host (default `127.0.0.1`) |
| `THUMBOR_DB_PORT` | port (backend-specific default) |
| `THUMBOR_DB_NAME` | database name (default `thumbor`) |
| `THUMBOR_DB_USERNAME` | optional username |
| `THUMBOR_DB_PASSWORD` | optional password |
| `THUMBOR_DB_PATH` | SQLite file path (default `thumbor.db`) |

Invalid `THUMBOR_DB_PORT` MUST warn and keep the previous value.

### Requirement: DbProvider abstraction

All backends MUST implement `db::DbProvider` with `ping`. SQL backends expose
`sql_pool()`; MongoDB exposes `mongo_client()`. The `db/` module establishes
connections only — entity queries live in `entity/repositories/`.

#### Scenario: integration tests use sqlite memory

- GIVEN tests call `tests/common::connect_state` with unique in-memory DB names
- WHEN `AppState::connect` runs in tests
- THEN startup succeeds without file collisions under parallel test execution

### Requirement: Entity migration at startup

For SQL backends, `AppState::connect` MUST call `entity::migrate` to ensure
`accounts` and `casbin_rule` tables exist before Casbin and login run.

MongoDB connections MUST NOT run entity migration; auth features require a
relational backend.

See `entity` spec for table definitions and repository contracts.

### Requirement: Startup is mandatory

`AppState::connect` MUST connect the database; connection failure MUST prevent
the server from starting with an `AppError`.
