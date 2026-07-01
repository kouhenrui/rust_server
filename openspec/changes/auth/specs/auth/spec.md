# Authentication (delta)

## ADDED Requirements (phase 1)

See canonical spec at `openspec/specs/auth/spec.md`.

## ADDED Requirements (phase 2 — planned)

### Requirement: Login endpoint

The service MUST expose `POST /login` accepting JSON credentials and returning
a JWT in the unified success envelope on success.

#### Scenario: valid login

- GIVEN a registered user with a bcrypt password hash in the database
- WHEN `POST /login` with correct username and password
- THEN HTTP 200, `data.token` is a verifiable JWT

#### Scenario: invalid credentials

- GIVEN wrong password
- WHEN `POST /login`
- THEN HTTP 401 and `err.kind` is `unauthorized`
