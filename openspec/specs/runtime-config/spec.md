# Runtime Configuration

## Purpose

Define the environment variables that control runtime behavior of the service.

## Requirements

### Requirement: Primary prefix

Service-owned configuration variables MUST use the `THUMBOR_` prefix. The
logging subsystem additionally reads `RUST_LOG` (standard tracing convention).

#### Scenario: THUMBOR variables loaded

- GIVEN the service is starting
- WHEN it loads `Config`, cache config, or database config
- THEN it reads variables documented below with the `THUMBOR_` prefix

### Requirement: Configurable bind address

The service MUST bind to the address given by `THUMBOR_BIND` (default
`0.0.0.0:8080`).

#### Scenario: bind to override

- GIVEN `THUMBOR_BIND=127.0.0.1:9000`
- WHEN the service starts
- THEN the listening socket is bound to 127.0.0.1:9000

### Requirement: Invalid values fall back to defaults

When a `THUMBOR_*` variable has an invalid value, the service MUST log a
warning via the encapsulated `warn!` macro and fall back to the documented
default. The service MUST NOT refuse to start because of a malformed value.

#### Scenario: bad bind address

- GIVEN `THUMBOR_BIND=not-an-addr`
- WHEN the service starts
- THEN startup succeeds, a warning is logged, and the bind address is the
  default `0.0.0.0:8080`

### Requirement: Core service variables

The service MUST read at least these variables:

| variable | default | purpose |
|---|---|---|
| `THUMBOR_BIND` | `0.0.0.0:8080` | listen address |
| `THUMBOR_MAX_SOURCE_BYTES` | `26214400` | cap on source image size |
| `THUMBOR_FETCH_TIMEOUT_MS` | `10000` | remote fetch timeout |
| `THUMBOR_WATERMARK_FONT` | _unset_ | path to a TTF for text watermarks |
| `THUMBOR_ALLOW_REMOTE` | `true` | allow `http(s)://` sources |
| `THUMBOR_LOCAL_SOURCE_ROOT` | _unset_ | base for relative local sources |
| `THUMBOR_LOG_LEVEL` | _unset_ | fallback log level when `RUST_LOG` absent |

#### Scenario: defaults match the documentation

- GIVEN no `THUMBOR_*` environment variables are set
- WHEN the service starts
- THEN the bind address is `0.0.0.0:8080`, the source size cap is
  26214400 bytes, the fetch timeout is 10000 ms, remote sources are allowed,
  and there is no font or local source root

### Requirement: Dotenv loading

The binary MUST load a `.env` file via `dotenvy` before logger
initialization so `RUST_LOG` and `THUMBOR_*` values are available at startup.

#### Scenario: dotenv before tracing

- GIVEN a `.env` file sets `RUST_LOG=debug`
- WHEN `main` starts
- THEN `Config::load_dotenv()` runs before `logger::init()`
