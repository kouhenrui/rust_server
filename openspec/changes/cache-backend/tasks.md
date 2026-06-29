# Tasks: cache-backend

## 1. Cache module

- [x] 1.1 Create `src/cache/` with `config.rs`, `cache.rs`, `memory.rs`,
  `redis.rs`. (`src/cache/`)
- [x] 1.2 Define `Cache` trait: `get`, `set`, `delete`, `ping`, `backend_name`.
- [x] 1.3 Implement `NoopCache` for disabled backend.

## 2. Configuration

- [x] 2.1 `CacheBackendConfig::from_env()` reads `THUMBOR_CACHE_BACKEND`.
- [x] 2.2 Memory: `THUMBOR_CACHE_MEMORY_MAX_ENTRIES`,
  `THUMBOR_CACHE_MEMORY_TTL_SECS`.
- [x] 2.3 Redis: URL or discrete host/port/db/auth vars with redacted logging.

## 3. Startup wiring

- [x] 3.1 `AppState::connect` calls `Cache::connect` and logs backend name.

## 4. Tests

- [x] 4.1 Unit tests: disabled no-op, memory roundtrip. (`src/cache/cache.rs`)

## 5. Verification

- [x] 5.1 `cargo test --lib` passes.
