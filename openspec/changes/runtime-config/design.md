# Design: runtime-config

## Context

This change owns the configuration-loading step at service startup.
Configuration drives the bind address, the source-handling
constraints, the font path, and the remote-source toggle. Other
changes depend on this one for the values they read.

## Goals

- One prefix (`THUMBOR_`) for all config.
- Defaults that are safe for local development and acceptable for
  production (bind on all interfaces, allow remote, 25 MiB cap,
  10s fetch timeout).
- Misconfiguration must not crash the service — log and fall back.

## Non-goals (recap)

No file config, no live reload, no per-request overrides, no
secrets. See `proposal.md`.

## Decisions

### Env vars only, no config file

We read configuration from `std::env::var` exclusively. There is no
YAML/TOML/JSON file. The reason is operator ergonomics in
containerized deployments: an env var is a one-line change in a
Helm chart or a `docker run`, and the operator can list every
configuration knob with `docker inspect` or `kubectl describe`.

**Trade-off:** long values (e.g. a multi-line JSON config) are
inconvenient in env vars. None of the variables in scope are that
long, and "if you need it, it's not a config value, it's a
fixture" holds in practice.

### `Config::default()` first, then overlay env vars

`from_env` starts with `Config::default()` and replaces fields one
at a time when the corresponding `THUMBOR_*` is set. The default is
always reachable; the env var is a delta.

**Why:** the default is the "documented behavior when nothing is
set" — keeping it in `Config::default()` is a single source of
truth. The env-var loop is then "for each variable, parse and
overlay". Adding a new variable is two places: a `pub` field on
`Config` and a branch in `from_env`.

### Invalid values log a warning, do not error

A `THUMBOR_BIND=not-an-addr` does not crash the service. It
produces a `crate::warn!` and the field stays at the default.
The trade-off is explicit: we have decided that a misconfigured
deployment is a recoverable problem, not a fatal one.

**Why:** the alternative (fail to start) means an operator
debugging a config typo has to look at the logs *and* redeploy.
Warn-and-fall-back means they see the warning in the logs and the
service is at least running with defaults — they can still hit
`/health` and `/img?src=cat.png` while they fix the typo.

### "True" accepts `1` / `true` / `yes`, case-insensitive

`THUMBOR_ALLOW_REMOTE` accepts any of `1`, `true`, `yes`
(case-insensitive). We did not restrict to exactly `true`/`false`.

**Why:** Helm value files and Kubernetes ConfigMaps routinely
contain `True` or `YES` (capitalization varies). Constraining to
`true` is hostile to the operator; the small extra parse logic is
worth the fewer "I set the var to True and it didn't take" bug
reports.

### `from_env` is `pub fn` returning `Self`, not `Result`

`Config::from_env` does not return `Result`. Misconfigurations
fall back to defaults, so the function can't fail.

**Why:** the caller (`main.rs`) is then a simple `let config =
Config::from_env();` with no `?`. The cost is a less-explicit API
contract ("from_env can't fail" is a property of the function, not
a type-level guarantee). The behavior is documented in the
function's docstring.

### `THUMBOR_FETCH_TIMEOUT_MS` is milliseconds, not seconds

The timeout env var is named `THUMBOR_FETCH_TIMEOUT_MS` and the
value is in milliseconds.

**Why:** the smallest reasonable timeout is ~100ms (otherwise
network jitter alone causes failures); the largest is 60s+ for
deliberately slow upstreams. Encoding in seconds would lose the
lower end; encoding in milliseconds keeps the range useful.

## File map

- `src/config.rs` — `Config`, `Default::default`, `from_env`. Added
  in this change.

## Open questions

- Should `THUMBOR_*` variables be validated against a JSON schema
  at startup, with a clear "this is the wrong value, here's what
  we expected" message? Not in this change — the current
  `warn_and_fall_back` behavior is enough.
