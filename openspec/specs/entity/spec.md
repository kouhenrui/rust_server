# Entity Layer

## Purpose

Define SQL entity tables, migrations, and the model/repository layering used
by authentication and Casbin policy storage.

## Architecture

| Layer | Path | Responsibility |
|-------|------|----------------|
| Model | `entity/models/` | Struct definitions, domain helpers (`is_active`) |
| Schema | `entity/schema.rs` | DDL per `SqlBackend` dialect |
| Repository | `entity/repositories/` | SQL CRUD and queries |
| Dialect | `entity/sql_backend.rs` | `accounts_upsert_sql`, `casbin_insert_sql` |

Business logic (password verify, JWT) stays in `auth/`; HTTP stays in
`controller/`.

## Requirements

### Requirement: Supported SQL backends

Entity tables MUST work on PostgreSQL, MySQL, and SQLite via sqlx `AnyPool`.
MongoDB MUST NOT be used for entity tables; `SqlBackend::require_from_db`
returns an explicit error for non-relational backends.

#### Scenario: migrate on sqlite

- GIVEN an empty in-memory SQLite pool
- WHEN `entity::migrate(pool, SqlBackend::Sqlite)` runs
- THEN tables `accounts` and `casbin_rule` exist

### Requirement: accounts table

The `accounts` table MUST store:

| column | notes |
|--------|-------|
| `id` | autoincrement primary key |
| `username` | unique, JWT `sub` |
| `password_hash` | bcrypt |
| `email`, `phone`, `nickname` | optional profile fields |
| `status` | `active` / `disabled` / `locked` |
| `last_login_at` | updated on successful login |
| `created_at`, `updated_at` | timestamps |
| `deleted_at` | soft delete; `NULL` = active row |

#### Scenario: login lookup excludes deleted

- GIVEN a row with `deleted_at` set
- WHEN `AccountRepository::find_auth_by_username` runs
- THEN the row is not returned

### Requirement: AccountRepository

SQL access for accounts MUST go through `entity::AccountRepository`:

- `find_auth_by_username` — login projection (`id`, `password_hash`, `status`)
- `upsert` — insert or update hash/status (bootstrap/tests)
- `touch_last_login` — set `last_login_at` after successful auth

Dialect-specific upsert SQL MUST live on `SqlBackend::accounts_upsert_sql()`.

#### Scenario: upsert via repository

- GIVEN a migrated SQLite pool and a bcrypt password hash
- WHEN `AccountRepository::upsert` is called with username `alice`
- THEN `find_auth_by_username("alice")` returns the stored hash and active status

### Requirement: casbin_rule table

Casbin policies MUST persist in `casbin_rule` with columns `ptype`, `v0`–`v5`
and a unique constraint on the full rule tuple. Policies MUST NOT be loaded
from CSV at runtime.

#### Scenario: seed when empty

- GIVEN `casbin_rule` has zero rows
- WHEN `CasbinAuth::new` runs
- THEN default RBAC policies from `auth/casbin_db.rs` are inserted

### Requirement: CasbinRuleRepository

SQL access for Casbin MUST go through `entity::CasbinRuleRepository` (used by
`auth/casbin_adapter.rs` and `auth/casbin_db.rs`).

#### Scenario: list policies ordered

- GIVEN seeded Casbin policies in `casbin_rule`
- WHEN `CasbinRuleRepository::list_all_ordered` runs
- THEN rows are returned sorted by `id`

### Requirement: Test utilities

Lib unit tests MUST use `entity::test_util::migrated_pool(name)` for a
migrated in-memory SQLite pool. Integration tests MUST use
`tests/common/mod.rs` with per-run unique database names to avoid parallel
test collisions.

#### Scenario: parallel integration tests

- GIVEN two integration tests run concurrently
- WHEN each calls `tests/common::connect_state`
- THEN each uses a distinct in-memory SQLite database name
