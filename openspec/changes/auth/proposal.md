# Change: auth

## Why

The service needs password hashing and JWT for future login-protected APIs.
Crypto logic belongs in `src/auth/`, not `util/`.

## What Changes (phase 1 — done)

- `auth/password.rs` — bcrypt hash/verify
- `auth/jwt.rs` — `JwtAuth`, `Claims`, `bearer_token`
- `Config` — `jwt_secret`, `jwt_expire_secs` env vars
- `AppState.jwt` — shared signer/verifier
- `AppError` — `Unauthorized`, `InvalidToken` (401)

## What Changes (phase 2 — planned)

- `POST /login` returning unified JSON envelope with token
- `auth_middleware` validating `Authorization: Bearer`
- User model + db persistence (sqlx)

## Capabilities

- `auth` — password + JWT library and future HTTP auth

## Non-goals

- OAuth2 / OIDC providers
- Refresh-token rotation (initial version)
- Per-tenant RBAC

## Affected domains

- **auth** (created)
- **runtime-config** (JWT env vars)
- **http-api** (future `/login` route)
