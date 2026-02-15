---
date: 2026-02-14
topic: "egs-api quality improvements"
status: validated
---

# egs-api Quality Improvements Design

## Problem Statement

The egs-api library has accumulated technical debt across HTTP handling, error management, and binary manifest parsing. Key issues: ~200 lines of duplicated HTTP boilerplate, panics in a library crate (unwrap on URLs, missing fields, truncated buffers), HTTP client rebuilt per-request defeating connection pooling, and a 1059-line monolithic manifest parser.

## Constraints

- **No semver-breaking changes in items 1-5** — this is a published crate (v0.8.1)
- **Edition 2018** — no 2021+ features
- **Must pass existing CI** — `cargo build --lib && cargo test --tests --lib`
- **`#![deny(missing_docs)]`** — all new public items need doc comments
- **`#![cfg_attr(test, deny(warnings))]`** — zero warnings in tests

## Improvements Overview

### NOW — Items 1-5 (High impact, non-breaking, execute immediately)

| # | Improvement | What Changes |
|---|------------|--------------|
| 1 | Extract HTTP helper methods | New `send_get_json<T>()`, `send_post_json<T>()`, `send_get_bytes()` internal methods on `EpicAPI` in `mod.rs` |
| 2 | Reuse `self.client` | `authorized_get_client`/`authorized_post_client` stop calling `build_client()`, use `self.client` instead |
| 3 | Fix `to_vec()` panic on `chunk_sha_list: None` | Handle `None` case with empty iteration instead of `unwrap()` |
| 4 | Fix `files.resize()` bug in `to_vec()` | Replace `files.resize()` with correct flag byte appending |
| 5 | Replace `unwrap()` with `?` on URLs + parsing | Convert panicking unwraps to `Result` propagation in internal methods |

### LATER — Items 6-12 (Medium/low impact or breaking)

| # | Improvement | What Changes |
|---|------------|--------------|
| 6 | Decompose `from_vec()`/`to_vec()` | Split into `parse_header/meta/chunks/files/custom_fields` functions |
| 7 | Enrich `EpicAPIError` with source errors | Add `NetworkError(reqwest::Error)`, `DeserializationError(String)`, `HttpError { status, body }` variants |
| 8 | Add `BinaryReader`/`BinaryWriter` abstraction | New internal structs wrapping `&[u8]` / `Vec<u8>` with bounds-checked reads |
| 9 | Add `Result`-returning facade methods | New `try_*` methods on `EpicGames` alongside existing `Option`-returning ones |
| 10 | Fix `Default` derive footgun | Implement `Default` manually for `EpicAPI` to call `new()`, or remove derive |
| 11 | Reduce `.clone()` usage | Change API methods to take `&EpicAsset` instead of owned, iterate by reference in hot paths |
| 12 | Add manifest parsing tests | Unit tests for `from_vec()` and `to_vec()` with sample binary/JSON data |

---

## Detailed Design — Items 1-5

### Item 1: Extract HTTP Helper Methods

**Location:** `src/api/mod.rs`

**New internal methods on `EpicAPI`:**

- `async fn send_get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, EpicAPIError>` — builds authorized GET, sends, checks status, parses JSON, maps errors
- `async fn send_post_json<T: DeserializeOwned, B: Serialize>(&self, url: &str, body: &B) -> Result<T, EpicAPIError>` — same for POST with JSON body
- `async fn send_post_form<T: DeserializeOwned>(&self, url: &str, form: &[(&str, String)]) -> Result<T, EpicAPIError>` — POST with form encoding (for ownership_token)
- `async fn send_get_bytes(&self, url: &str) -> Result<Vec<u8>, EpicAPIError>` — for manifest binary downloads

**Error mapping inside helpers:**
- URL parse failure → `EpicAPIError::InvalidParams`
- Network error → `EpicAPIError::Unknown` (logged with `error!`)
- Non-200 status → `EpicAPIError::Unknown` (logged with `warn!`, body logged)
- 403 status → check if Fab context, map to `FabTimeout` where appropriate
- JSON parse error → `EpicAPIError::Unknown` (logged with `error!`)

**Then refactor callers:** `egs.rs`, `account.rs`, `fab.rs` methods become 2-5 line functions that build a URL and call the helper. `login.rs` keeps its own flow (basic_auth, form params, special response handling).

**Special cases:**
- `fab_asset_manifest` needs 403→`FabTimeout` mapping — add an optional status-code hook or handle it at the call site after the helper
- `asset_download_manifests` fetches without auth — add `send_get_bytes_unauth()` or use `self.client` directly
- `library_items` / `fab_library_items` have pagination loops — they call the helper repeatedly, loop logic stays in the method

### Item 2: Reuse `self.client`

**Location:** `src/api/mod.rs`

**Change `authorized_get_client` and `authorized_post_client`:**
- Remove: `let client = EpicAPI::build_client().build().unwrap();`
- Replace with: `self.client.get(url)` / `self.client.post(url)`
- Keep: `self.set_authorization_header(...)` call

**Change unauthenticated requests** in `egs.rs::asset_download_manifests`, `fab.rs::fab_download_manifest`, `login.rs::invalidate_sesion`:
- Remove: `let client = EpicAPI::build_client().build().unwrap();`
- Replace with: `self.client.get(url)` / `self.client.delete(url)` (without auth header)

**Benefit:** Connection pooling works, cookies persist across requests (important for session), fewer allocations.

**Risk:** If any endpoint relied on fresh cookie state — unlikely and undesirable. Test login + subsequent API calls.

### Item 3: Fix `to_vec()` Panic on `chunk_sha_list: None`

**Location:** `src/api/types/download_manifest.rs`, in `to_vec()` method

**Current code (panics):**
```
for sha in self.chunk_sha_list.as_ref().unwrap().values() {
```

**Fix:** Handle `None` by writing zero-filled SHA entries:
```
match &self.chunk_sha_list {
    Some(sha_list) => { iterate sha_list.values(), decode_hex and append }
    None => { for each chunk in chunk_hash_list.keys(), append 20 zero bytes }
}
```

### Item 4: Fix `files.resize()` Bug in `to_vec()`

**Location:** `src/api/types/download_manifest.rs`, in `to_vec()` method

**Current code (likely truncates):**
```
// flags
// TODO: Figure out what Epic puts in theirs
files.resize(self.file_manifest_list.len(), 0);
```

This resizes the entire `files` Vec to `file_manifest_list.len()` — which is almost certainly smaller than its current size (it already has filenames, symlinks, hashes). This **truncates** the buffer.

**Fix:** Append one zero byte per file for the flags field:
```
for _ in &self.file_manifest_list {
    files.push(0u8);
}
```

### Item 5: Replace `unwrap()` with Error Propagation

**Locations and fixes:**

1. **`src/api/mod.rs` — `authorized_get_client` / `authorized_post_client`:**
   - `Url::parse(&url).unwrap()` → callers should pass `&str`, helpers parse with `?`

2. **`src/api/egs.rs`:**
   - `Url::parse(&url).unwrap()` in `assets`, `asset_manifest`, `asset_info`, `game_token`, `ownership_token` → use `?` with mapping to `InvalidParams`
   - `response.text().await.unwrap()` in warn branches → use `.unwrap_or_default()` or `match`

3. **`src/api/account.rs`:**
   - Same `Url::parse().unwrap()` and `response.text().await.unwrap()` patterns

4. **`src/api/fab.rs`:**
   - Same patterns
   - `response.text().await.unwrap()` on OK path in `fab_asset_manifest`

5. **`src/api/login.rs`:**
   - `self.user_data.refresh_token.clone().unwrap()` in `start_session` → return `Err(EpicAPIError::InvalidCredentials)` if refresh token is `None`
   - `Url::from_str(&url).unwrap()` in `invalidate_sesion`

6. **`src/api/types/download_manifest.rs`:**
   - `z.read_to_end(&mut data).unwrap()` → return `None`
   - `buffer[position]` direct accesses → bounds check (or defer to item 8)
   - `Url::parse()` in `download_links()` → return `None` for invalid URLs

**Strategy:** Since internal methods already return `Result<T, EpicAPIError>`, most `unwrap()` calls can be replaced with `?` after mapping the error. For `download_manifest.rs` where the return type is `Option`, use `.ok()?` or early return `None`.

Note: Item 5 for `download_manifest.rs` should focus on the **most dangerous panics** only (URL parsing, zlib decompression, missing chunk_sha_list). The comprehensive bounds-checking refactor is deferred to items 6+8.

---

## Testing Strategy

- **All items:** Must pass `cargo build --lib && cargo test --tests --lib`
- **Items 1-2:** Behavioral — same API surface, same results. Existing tests + manual verification.
- **Items 3-4:** Bug fixes — ideally add a test that exercises `to_vec()` on a JSON-parsed manifest (chunk_sha_list: None) and verifies output isn't truncated. But existing test infrastructure is minimal, so at minimum verify `cargo test` passes.
- **Item 5:** Each replaced `unwrap()` should handle the error gracefully instead of panicking.

## Open Questions

None — items 1-5 are straightforward and non-breaking. Items 6-12 have design decisions (e.g., error enum variants, whether to split files) that can be resolved when they're picked up.
