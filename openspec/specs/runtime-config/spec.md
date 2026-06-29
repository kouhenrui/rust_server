# Runtime Configuration

## Purpose
Define the environment variables that control runtime behavior of the service.

## Requirements

### Requirement: Single prefix
All environment variables read by the service MUST start with the `THUMBOR_`
prefix.

#### Scenario: no other prefix is read
- GIVEN the service is starting
- WHEN it loads configuration
- THEN it reads only variables beginning with `THUMBOR_`

### Requirement: Configurable bind address
The service MUST bind to the address given by `THUMBOR_BIND` (default
`0.0.0.0:8080`).

#### Scenario: bind to override
- GIVEN `THUMBOR_BIND=127.0.0.1:9000`
- WHEN the service starts
- THEN the listening socket is bound to 127.0.0.1:9000

### Requirement: Invalid values fall back to defaults
When an environment variable has an invalid value, the service MUST log a
warning and fall back to the documented default. The service MUST NOT refuse
to start because of a malformed configuration value.

#### Scenario: bad bind address
- GIVEN `THUMBOR_BIND=not-an-addr`
- WHEN the service starts
- THEN startup succeeds, a warning is logged, and the bind address is the
  default `0.0.0.0:8080`

### Requirement: Documented variables
The service MUST read at least these variables, with the defaults documented
in `AGENTS.md`:

| variable | purpose |
|---|---|
| `THUMBOR_BIND` | listen address |
| `THUMBOR_MAX_SOURCE_BYTES` | cap on source image size |
| `THUMBOR_FETCH_TIMEOUT_MS` | remote fetch timeout |
| `THUMBOR_WATERMARK_FONT` | path to a TTF for text watermarks |
| `THUMBOR_ALLOW_REMOTE` | allow `http(s)://` sources |
| `THUMBOR_LOCAL_SOURCE_ROOT` | base for relative local sources |

#### Scenario: defaults match the documentation
- GIVEN no `THUMBOR_*` environment variables are set
- WHEN the service starts
- THEN the bind address is `0.0.0.0:8080`, the source size cap is
  26214400 bytes, the fetch timeout is 10000 ms, remote sources are allowed,
  and there is no font or local source root
