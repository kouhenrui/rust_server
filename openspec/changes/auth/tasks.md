# Tasks: auth

## 1. Password hashing (phase 1)

- [x] 1.1 Add `bcrypt` dependency and `src/auth/password.rs` with
  `hash_password` / `verify_password`. (`src/auth/password.rs`)
- [x] 1.2 Reject empty passwords with `AppError::BadRequest`.
- [x] 1.3 Unit tests: roundtrip and empty rejection.

## 2. JWT library (phase 1)

- [x] 2.1 Add `jsonwebtoken` dependency and `src/auth/jwt.rs`.
- [x] 2.2 Define `Claims { sub, iat, exp }` and `JwtAuth::sign` / `verify`.
- [x] 2.3 Implement `bearer_token` header parser.
- [x] 2.4 Unit tests: sign/verify roundtrip, invalid token, bearer parse.

## 3. Configuration (phase 1)

- [x] 3.1 Add `jwt_secret`, `jwt_expire_secs` to `Config`. (`src/config.rs`)
- [x] 3.2 Load `THUMBOR_JWT_SECRET`, `THUMBOR_JWT_EXPIRE_SECS` in `from_env`.
- [x] 3.3 Document vars in `.env.example`.

## 4. AppState wiring (phase 1)

- [x] 4.1 Add `jwt: JwtAuth` to `AppState`, built in `connect`. (`src/state.rs`)
- [x] 4.2 Export auth types from `src/lib.rs`.

## 5. Error types (phase 1)

- [x] 5.1 Add `Unauthorized`, `InvalidToken` to `AppError` → 401.
  (`src/error.rs`)

## 6. Login HTTP API (phase 2)

- [x] 6.1 Add `POST /login` in `src/router.rs` → `controller/auth.rs`.
- [x] 6.2 Request body: `{ "username", "password" }` (JSON).
- [x] 6.3 Success: `{ code: 0, data: { token, expires_at } }`.
- [x] 6.4 Failure: wrong credentials → 401 `unauthorized`.
- [x] 6.5 Integration test in `tests/integration.rs`.

## 7. JWT middleware (phase 2)

- [x] 7.1 Add `src/middleware/auth.rs` — validate `Bearer` via `state.jwt`.
- [x] 7.2 Inject verified `Claims` into request extensions.
- [x] 7.3 Protect `GET /me` (public: `/health`, `/login`, `/img`).

## 8. User persistence (phase 2)

- [x] 8.1 Define users table migration in `src/auth/user.rs` (sql backends).
- [ ] 8.2 Store bcrypt hash on registration (future `POST /register`).
- [x] 8.3 Login looks up user and calls `verify_password`.

## 9. Production hardening (phase 2)

- [x] 9.1 Startup `warn!` when `jwt_secret` is still default `secret`.
- [x] 9.2 Login scenarios in `openspec/specs/auth/spec.md`.

## 10. Verification

- [x] 10.1 `cargo test` passes auth unit tests (phase 1).
- [x] 10.2 Integration tests cover login + protected route (phase 2).
