# PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-15
**Commit:** b552708 (uncommitted changes on top)
**Branch:** master

## OVERVIEW

Async Rust client library for the Epic Games Store API. Provides authentication, asset listing, download manifest parsing (binary + JSON), and Fab marketplace integration via `reqwest`/`tokio`.

## STRUCTURE

```
egs-api/
├── src/
│   ├── lib.rs                    # Public facade: EpicGames struct + facade_tests
│   └── api/
│       ├── mod.rs                # Internal EpicAPI struct + HTTP helpers (authorized_get_json, etc.)
│       ├── login.rs              # OAuth session: start, resume, invalidate
│       ├── egs.rs                # EGS methods: assets, manifests, library (uses HTTP helpers)
│       ├── account.rs            # Account details, friends, entitlements (uses HTTP helpers)
│       ├── fab.rs                # Fab marketplace: manifests, library (partial helper use)
│       ├── error.rs              # EpicAPIError enum + tests
│       ├── utils.rs              # Binary parsing helpers + tests (24 tests)
│       └── types/
│           ├── download_manifest.rs  # Binary manifest parser/serializer + tests (12 tests)
│           ├── account.rs            # UserData (session state), AccountData + tests (6 tests)
│           ├── asset_info.rs         # AssetInfo with release sorting + tests (9 tests)
│           ├── fab_library.rs        # FabLibrary with pagination cursors
│           ├── fab_asset_manifest.rs # Fab download info + distribution points + tests (3 tests)
│           ├── chunk.rs              # Downloaded chunk with decompression + tests (4 tests)
│           ├── asset_manifest.rs     # AssetManifest with URL CSV parsing
│           ├── epic_asset.rs         # EpicAsset (catalog item reference)
│           ├── library.rs            # Library with pagination metadata
│           ├── entitlement.rs        # Entitlement record
│           └── friends.rs            # Friend record
├── examples/
│   ├── common.rs                 # Shared auth helper (token persistence at ~/.egs-api/token.json)
│   ├── auth.rs                   # Interactive login + token persistence
│   ├── account.rs                # Account details, ID lookup, friends
│   ├── entitlements.rs           # List all entitlements
│   ├── library.rs                # Paginated library listing
│   ├── assets.rs                 # Full pipeline: list → info → manifest → download manifest
│   ├── game_token.rs             # Exchange code + ownership token (Fortnite hardcoded)
│   ├── fab.rs                    # Fab library → asset manifest → download manifest
│   └── workflow.rs               # Original auth flow demo (pre-existing)
├── Cargo.toml                    # v0.8.1, edition 2018, MIT, autoexamples=false
├── README.md                     # Comprehensive docs with badges, auth flow, endpoint tables
└── .github/workflows/rust.yml    # CI: cargo build --lib && cargo test --tests --lib
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new API endpoint | `src/api/egs.rs` or new file in `src/api/` | Impl block on `EpicAPI`, expose via `lib.rs` |
| Add new Fab endpoint | `src/api/fab.rs` | Same pattern as `fab_asset_manifest` |
| Add response type | `src/api/types/` | New file + re-export in `types/mod.rs` |
| Fix auth/session bugs | `src/api/login.rs` | OAuth token flow, refresh logic |
| Fix manifest parsing | `src/api/types/download_manifest.rs` | Binary format: header → meta → chunks → files → custom fields |
| Add public API method | `src/lib.rs` | Thin wrapper delegating to `self.egs.*` |
| Add tests | Same file as code, `#[cfg(test)] mod tests` | 68 tests across 8 files currently |
| Modify HTTP headers | `src/api/mod.rs` → constructor in `new()` | User-Agent, Correlation-ID |
| Add HTTP endpoint | `src/api/mod.rs` helpers | Use `authorized_get_json`, `authorized_post_form_json`, `get_bytes` |
| Add examples | `examples/` + `Cargo.toml` `[[example]]` | Must use `#[path = "common.rs"] mod common;` pattern |

## ARCHITECTURE

**Facade pattern**: `EpicGames` (public) wraps `EpicAPI` (pub(crate)).

- `EpicGames` in `lib.rs` — consumer-facing, returns `Option`/`Vec` (swallows errors), Fab methods return `Result`
- `EpicAPI` in `api/mod.rs` — internal, returns `Result<T, EpicAPIError>`
- API methods split across files via `impl EpicAPI` blocks in `login.rs`, `egs.rs`, `account.rs`, `fab.rs`

**HTTP client**: Single `reqwest::Client` built in `EpicAPI::new()` with cookie store, reused across all requests. Authorization via bearer token in `set_authorization_header()`.

**HTTP helpers** (on `EpicAPI`):
- `authorized_get_json<T>(&self, url)` — authorized GET → deserialize JSON
- `authorized_post_form_json<T>(&self, url, form)` — authorized POST with form data → deserialize JSON
- `get_bytes(&self, url)` — unauthenticated GET → raw bytes

**Auth flow**: `start_session()` → exchange/auth code/refresh token → `handle_login_response()` → updates `UserData`. Session resume via `/oauth/verify`. 600-second expiry threshold for re-login.

## CONVENTIONS

- `#![deny(missing_docs)]` — all public items require doc comments
- `#![cfg_attr(test, deny(warnings))]` — zero warnings in test builds
- `#[allow(missing_docs)]` on type structs with many fields
- `#[serde(rename_all = "camelCase")]` for JSON API responses
- `#[serde(rename_all = "PascalCase")]` for Epic's manifest format
- Custom serde deserializers for Epic's blob number format (`deserialize_epic_string`, `deserialize_epic_hash`)
- Error handling: API methods return `Result<T, EpicAPIError>`, facade methods return `Option<T>`
- No external test framework — inline `#[cfg(test)]` modules only
- Edition 2018 — no 2021+ features
- Manual `impl Default` for `EpicAPI` and `EpicGames` (delegates to `new()` for proper HTTP client init)
- Examples use `#[path = "common.rs"] mod common;` for shared auth (Cargo limitation)
- Token persistence: `~/.egs-api/token.json` (UserData serialized as JSON)

## ANTI-PATTERNS (THIS PROJECT)

- **Do not add `as any` equivalents** — no `unsafe` blocks exist, keep it that way
- **Do not change the User-Agent string** — Epic API may reject non-launcher user agents
- **5 TODOs in `download_manifest.rs`**: chunk ordering (line ~781), wrong uncompressed size (line ~807, known bug), 3x unknown Epic format fields (lines ~857, ~871, ~875)
- **Hardcoded credentials**: client_id/secret in `login.rs` (`34a02cf8f4414e29b15921876da36f9a`), correlation ID in `mod.rs` — these are Epic's public launcher credentials, not secrets
- **No async tests** — despite all API methods being async
- **`invalidate_sesion`** — typo in method name (missing 's'), preserved for API compat

## TEST SUITE

68 tests, all passing. Run: `cargo test --tests --lib`

| File | Tests | Coverage |
|------|-------|---------|
| `utils.rs` | 24 | `blob_to_num`, `bigblob_to_num`, `read_le` (all sizes), `read_fstring` (UTF-8/16/edge cases), `write_fstring`, `decode_hex`, `do_vecs_match` |
| `download_manifest.rs` | 12 | Binary round-trip (`from_vec` → `to_vec` → `from_vec`), `parse()` fallback (invalid binary/JSON), `chunk_dir()` version thresholds, custom fields get/set, `total_download_size`, `total_size`, `FileManifestList::size()` |
| `asset_info.rs` | 9 | `latest_release`, `sorted_releases`, `release_info(id)`, `release_name`, `compatible_apps` (dedup), `platforms` (aggregate), None cases |
| `error.rs` | 8 | `Display` for all 6 variants, `Error::description()`, `Debug` |
| `lib.rs` (facade_tests) | 7 | `new()`, `Default`, `user_details`, `set_user_details`, `is_logged_in` expired/valid/600s-threshold |
| `account.rs` | 6 | `UserData::new()` defaults, `update()` partial-merge (merges/preserves), serialization round-trip, access/refresh token getters |
| `chunk.rs` | 4 | `from_vec` valid uncompressed, valid compressed (zlib), wrong magic, version 2 SHA hash |
| `fab_asset_manifest.rs` | 3 | `get_distribution_point_by_base_url` found/not-found/empty |

## BINARY MANIFEST FORMAT

`download_manifest.rs` is the complexity hotspot. Binary format (little-endian):

```
Header (41 bytes): magic(u32) → header_size(u32) → size_uncompressed(u32) → size_compressed(u32) → sha_hash(20 bytes) → compressed(u8) → version(u32)
Body (zlib-compressed):
  Meta: data_version → manifest_version → is_file_data → app_id → strings...
  Chunks: version → count → [guid(16b) × N] → [hash(u64) × N] → [sha(20b) × N] → [group(u8) × N] → [window(u32) × N] → [filesize(i64) × N]
  Files: version → count → [filename × N] → [symlink × N] → [hash(20b) × N] → [flags(u8) × N] → [tags × N] → [chunk_parts × N]
  Custom Fields: version → count → [keys × N] → [values × N]
```

Parsing uses position-tracking via `&mut usize` with helpers in `utils.rs` (`read_le`, `read_fstring`, etc.).

## COMMANDS

```bash
cargo build --lib              # Build library
cargo test --tests --lib       # Run tests (CI command) — 68 tests
cargo build --examples         # Build all examples
cargo run --example auth       # Interactive auth demo (opens browser)
cargo doc --open               # Generate + view docs (may fail on recent toolchains)
```

## COMPLETED WORK (this session)

All changes are uncommitted, sitting on top of commit b552708.

### HTTP & Architecture
- Extracted HTTP helpers in `src/api/mod.rs`: `authorized_get_json<T>`, `authorized_post_form_json<T>`, `get_bytes`
- Inlined `build_client()` — constructor logic now directly in `new()`; single `self.client` reused everywhere
- Refactored callers: `egs.rs` (7 methods), `account.rs` (4 methods), `fab.rs` (2 of 3), `login.rs` (1 method)
- Fixed `Default` derive: Manual `impl Default` for `EpicAPI` and `EpicGames` delegating to `new()`

### Bug Fixes
- `to_vec()` panic on `chunk_sha_list: None` → `unwrap_or(&HashMap::new())`
- `to_vec()` truncation via `files.resize()` → `for _ { files.push(0u8) }`
- Dangerous unwraps: Zlib decompression and SHA hash conversions → graceful `return None`

### Examples (8 new + shared auth)
- `common.rs` (shared auth), `auth.rs`, `account.rs`, `entitlements.rs`, `library.rs`, `assets.rs`, `game_token.rs`, `fab.rs`
- All verified against live Epic Games API (100% pass rate)
- `Cargo.toml`: `autoexamples = false`, 8 explicit `[[example]]` entries

### Documentation
- README.md: Complete rewrite with badges, features, quick start, auth flow, endpoint tables, manifest format, architecture
- Cargo.toml: Added `keywords`, `categories`, `readme = "README.md"`
- `src/lib.rs`: Comprehensive `//!` crate-level docs with code example, enhanced struct/method docs
- Internal docs: Doc comments on all API methods and type definitions

### Test Suite (10 → 68 tests)
- `utils.rs` +14, `download_manifest.rs` +12, `chunk.rs` +4, `account.rs` +6
- `asset_info.rs` +9, `fab_asset_manifest.rs` +3, `error.rs` +8, `lib.rs` +7

## REMAINING BACKLOG

From `thoughts/shared/designs/2026-02-14-api-quality-improvements-design.md`, items 6-12:

| # | Improvement | Effort | Notes |
|---|------------|--------|-------|
| 6 | Decompose `download_manifest.rs` | High | Split `from_vec()`/`to_vec()` into `parse_header/meta/chunks/files/custom_fields` |
| 7 | Enrich `EpicAPIError` with source errors | Medium | Add `NetworkError(reqwest::Error)`, `HttpError { status, body }` — **semver-breaking** |
| 8 | Add `BinaryReader`/`BinaryWriter` abstraction | Medium | Bounds-checked reads, replaces raw `buffer[position]` indexing |
| 9 | Add `Result`-returning facade methods | Medium | New `try_*` methods on `EpicGames` alongside `Option`-returning ones |
| 11 | Reduce `.clone()` usage | Low | `&EpicAsset` instead of owned, iterate by reference |
| 12 | Add async integration tests | Medium | Would need mock HTTP server (e.g., `wiremock`) |

Items 10 (Default fix) and partial item 12 (sync tests) are already done.

## NOTES

- Published to crates.io as `egs-api`; docs at docs.rs/egs-api
- CI uses `actions/checkout@v2` (outdated) with no clippy/rustfmt/audit steps
- Fab API returns 403 on timeout → mapped to `EpicAPIError::FabTimeout`
- `UserData::update()` merges only `Some` fields — partial update pattern for token refresh
- `download_manifest.rs` supports both JSON and binary manifest formats; `parse()` tries binary first, falls back to JSON
- Pagination: `library_items` and `fab_library_items` use cursor-based loops
- `serde_with::DefaultOnNull` used in `fab_library.rs` to handle null arrays from API
- `fab_asset_manifest` kept lower-level (not using HTTP helpers) due to special 403→FabTimeout handling
- `game_token.rs` example has hardcoded Fortnite values: namespace `"fn"`, catalog_item `"4fe75bbc5a674f4f9b356b5c90567da5"`
- `cargo doc` may fail on recent Rust toolchains (1.93+) due to template engine bug — not our issue
- Auth URL: `https://www.epicgames.com/id/api/redirect?clientId=34a02cf8f4414e29b15921876da36f9a&responseType=code`
- Token file: `~/.egs-api/token.json` (UserData serialized as JSON)
