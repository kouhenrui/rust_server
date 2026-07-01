# Design: auth

## Decisions

### Separate `auth/` module

bcrypt and JWT carry security semantics and heavier dependencies. They stay
out of `util/` which remains stateless helpers.

### HS256 with shared secret

Symmetric signing keeps deployment simple for a single-service token issuer.
`THUMBOR_JWT_SECRET` must be overridden in production.

### JwtAuth on AppState

Handlers receive `State<Arc<AppState>>` and call `state.jwt.sign/verify` —
no global static secret.

### Error mapping

- `InvalidToken` — malformed/expired JWT (401 `invalid_token`)
- `Unauthorized` — missing auth or bad credentials (401 `unauthorized`)

## File map

- `src/auth/password.rs` — bcrypt
- `src/auth/jwt.rs` — jsonwebtoken
- `src/config.rs` — JWT settings
- `src/state.rs` — `jwt: JwtAuth`
- `src/error.rs` — auth error variants

## Planned (phase 2)

- `src/controller/auth.rs` — `POST /login`
- `src/middleware/auth.rs` — bearer validation layer
- `src/auth/user.rs` — db-backed user lookup (TBD)
