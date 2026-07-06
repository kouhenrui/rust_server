# Authentication

## Purpose

Define password hashing, JWT, SQL-backed account storage, and Casbin RBAC
used for login and route protection.

## Architecture

| Layer | Path | Responsibility |
|-------|------|----------------|
| Model | `entity/models/account.rs` | `Account`, `AccountAuth`, status constants |
| Repository | `entity/repositories/account.rs` | SQL CRUD for `accounts` table |
| Service | `auth/account.rs` | Login business (`authenticate`: bcrypt + `last_login_at`) |
| Controller | `controller/auth.rs` | `POST /login`, `GET /me`, bootstrap admin |

Casbin policies use `entity/repositories/casbin_rule.rs` (not CSV); model
file is `config/casbin_model.conf`.

## Requirements

### Requirement: Password hashing

The service MUST hash passwords with bcrypt (default cost 12) via
`auth::hash_password` and verify via `auth::verify_password`.

#### Scenario: hash and verify

- GIVEN a non-empty plaintext password
- WHEN `hash_password` then `verify_password` with the same plaintext
- THEN verification returns true

#### Scenario: empty password rejected

- GIVEN an empty plaintext password
- WHEN `hash_password` is called
- THEN the result is `AppError::BadRequest`

### Requirement: Account persistence

Accounts MUST be stored in the SQL `accounts` table (PostgreSQL / MySQL /
SQLite). MongoDB is not supported for auth tables.

Fields include `username` (unique), `password_hash`, `status`
(`active` / `disabled` / `locked`), soft-delete via `deleted_at`, and
`last_login_at` updated on successful login.

#### Scenario: authenticate updates last login

- GIVEN an active account with a valid bcrypt hash
- WHEN `auth::authenticate` succeeds
- THEN `last_login_at` is updated in the database

#### Scenario: disabled account rejected

- GIVEN an account with `status != active`
- WHEN `authenticate` is called with correct password
- THEN the result is `AppError::Unauthorized`

Repository access MUST go through `entity::AccountRepository`; bootstrap and
tests may call `AccountRepository::upsert` directly after `hash_password`.

### Requirement: JWT configuration

JWT signing MUST use `Config.jwt_secret` and `Config.jwt_expire_secs`, loaded
from `THUMBOR_JWT_SECRET` and `THUMBOR_JWT_EXPIRE_SECS`.

#### Scenario: env override

- GIVEN `THUMBOR_JWT_SECRET=prod-key` and `THUMBOR_JWT_EXPIRE_SECS=3600`
- WHEN `Config::from_env()` runs
- THEN `jwt_secret` is `prod-key` and `jwt_expire_secs` is 3600

### Requirement: JWT issue and verify

`JwtAuth` MUST sign tokens containing `Claims { sub, iat, exp }` and verify
HS256 tokens via `jsonwebtoken`.

#### Scenario: round trip

- GIVEN `JwtAuth::new(&config)` and subject `user-1`
- WHEN `sign` then `verify` on the returned token
- THEN `claims.sub` is `user-1`

#### Scenario: invalid token

- GIVEN a malformed or expired token string
- WHEN `verify` is called
- THEN the error is `AppError::InvalidToken` with HTTP 401

### Requirement: Bearer header parsing

`auth::bearer_token` MUST extract the token from `Authorization: Bearer <token>`.

#### Scenario: bearer prefix

- GIVEN header value `Bearer eyJhbG...`
- WHEN `bearer_token` is called
- THEN it returns `Some("eyJhbG...")`

### Requirement: AppState integration

`AppState` MUST hold `jwt: JwtAuth` and `casbin: CasbinAuth` built from
runtime config and SQL pool.

#### Scenario: state carries jwt helper

- GIVEN `AppState::connect` succeeds
- WHEN a handler accesses `state.jwt`
- THEN it can call `sign` / `verify` without reconstructing keys

### Requirement: Login endpoint

The service MUST expose `POST /login` accepting JSON `{ "username", "password" }`.
On success it MUST return `{ code: 0, data: { token, expires_at } }` where
`token` is a signed JWT and `expires_at` is a Unix timestamp.

#### Scenario: valid credentials

- GIVEN a user exists in the SQL database with a bcrypt password hash
- WHEN `POST /login` is called with the correct password
- THEN the response is HTTP 200 with a non-empty `data.token`

#### Scenario: invalid credentials

- GIVEN wrong username or password
- WHEN `POST /login` is called
- THEN the response is HTTP 401 with `err.kind` = `unauthorized`

### Requirement: Protected profile endpoint

`GET /me` MUST require a valid `Authorization: Bearer <token>` header and
return `{ code: 0, data: { username } }` from JWT `sub`.

#### Scenario: missing token

- GIVEN no `Authorization` header
- WHEN `GET /me` is called
- THEN the response is HTTP 401

### Requirement: Casbin RBAC authorization

All HTTP routes MUST pass through Casbin enforcement after authentication
resolution. The subject is the JWT `sub` when a valid Bearer token is present,
otherwise `anonymous` for public routes.

Model file defaults to `config/casbin_model.conf` (override via
`THUMBOR_CASBIN_MODEL`). Policies are stored in the SQL table `casbin_rule`
via `CasbinRuleRepository` and seeded on first startup when the table is empty.
Denied requests MUST return HTTP 403 with `err.kind` = `forbidden`.

#### Scenario: anonymous health

- GIVEN no Authorization header
- WHEN `GET /health` is called
- THEN Casbin allows the request

#### Scenario: role-based me access

- GIVEN user `testuser` has role `user` in Casbin policy
- WHEN `GET /me` is called with a valid token for `testuser`
- THEN Casbin allows the request

### Requirement: Bootstrap admin

The service MUST, when `THUMBOR_BOOTSTRAP_USERNAME` and
`THUMBOR_BOOTSTRAP_PASSWORD` are both non-empty, upsert the account and assign
Casbin role `admin` (idempotent). When either variable is unset, bootstrap MUST
be skipped.

#### Scenario: bootstrap on startup

- GIVEN both bootstrap env vars are non-empty
- WHEN the server starts
- THEN the user exists in `accounts` and has grouping policy `g, <username>, admin`
