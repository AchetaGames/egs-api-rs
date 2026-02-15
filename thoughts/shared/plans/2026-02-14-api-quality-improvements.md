---
date: 2026-02-14
design: thoughts/shared/designs/2026-02-14-api-quality-improvements-design.md
scope: "Items 1-5 only"
status: ready
---

# Implementation Plan: egs-api Quality Improvements (Items 1-5)

## Constraints Reminder

- No semver-breaking changes
- Edition 2018
- `#![deny(missing_docs)]` — doc comments on all new pub items
- `#![cfg_attr(test, deny(warnings))]` — zero warnings
- Must pass: `cargo build --lib && cargo test --tests --lib`

## Execution Order

Items ordered for minimal churn — each step builds on the previous.

---

## Step 1: Reuse `self.client` (Item 2)

**Files:** `src/api/mod.rs`

### 1a. Fix `authorized_get_client`

Change from:
```rust
fn authorized_get_client(&self, url: Url) -> RequestBuilder {
    let client = EpicAPI::build_client().build().unwrap();
    self.set_authorization_header(client.get(url))
}
```

To:
```rust
fn authorized_get_client(&self, url: Url) -> RequestBuilder {
    self.set_authorization_header(self.client.get(url))
}
```

### 1b. Fix `authorized_post_client`

Same pattern — replace `EpicAPI::build_client().build().unwrap()` with `self.client`.

### 1c. Fix unauthenticated requests

In these locations, replace `let client = EpicAPI::build_client().build().unwrap();` with `self.client`:

- `src/api/egs.rs` → `asset_download_manifests` method: `let client = EpicAPI::build_client().build().unwrap();` → `self.client` (note: this method takes `&self` so this works)
- `src/api/fab.rs` → `fab_download_manifest` method: same change
- `src/api/login.rs` → `invalidate_sesion` method: same change

### Verify

```bash
cargo build --lib && cargo test --tests --lib
```

---

## Step 2: Extract HTTP Helper Methods (Item 1)

**Files:** `src/api/mod.rs` (new methods), then `src/api/egs.rs`, `src/api/account.rs`, `src/api/fab.rs` (refactor callers)

### 2a. Add helper methods to `EpicAPI` in `src/api/mod.rs`

Add these internal helper methods inside the `impl EpicAPI` block:

**`authorized_get_json`** — Authorized GET that returns deserialized JSON:
- Signature: `pub(crate) async fn authorized_get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, EpicAPIError>`
- Implementation:
  1. Parse URL with `Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?`
  2. `self.authorized_get_client(parsed_url).send().await` — map Err to `EpicAPIError::Unknown` with `error!` log
  3. Check `response.status() == StatusCode::OK` — if not, `warn!` log status + body (using `.text().await.unwrap_or_default()`), return `Err(EpicAPIError::Unknown)`
  4. `response.json::<T>().await` — map Err to `EpicAPIError::Unknown` with `error!` log
  5. Return `Ok(result)`

**`authorized_post_json`** — Authorized POST with JSON body:
- Signature: `pub(crate) async fn authorized_post_json<T: serde::de::DeserializeOwned>(&self, url: &str, body: &impl serde::Serialize) -> Result<T, EpicAPIError>`
- Same pattern as above but uses `authorized_post_client` and `.json(body)`

**`authorized_post_form`** — Authorized POST with form body:
- Signature: `pub(crate) async fn authorized_post_form<T: serde::de::DeserializeOwned>(&self, url: &str, form: &[(String, String)]) -> Result<T, EpicAPIError>`
- Same but uses `.form(form)`

**`get_bytes`** — Unauthenticated GET returning raw bytes:
- Signature: `pub(crate) async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, EpicAPIError>`
- Uses `self.client.get(...)` (no auth header)
- Returns `response.bytes().await` as `Vec<u8>`

Add necessary imports at top of `mod.rs`: `use serde::de::DeserializeOwned;`

### 2b. Refactor `src/api/egs.rs`

Replace boilerplate in each method:

**`assets`** — Replace the entire match block with:
```rust
let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
self.authorized_get_json(&url).await
```

**`asset_manifest`** — After param validation and URL building:
```rust
let mut manifest: AssetManifest = self.authorized_get_json(&url).await?;
manifest.platform = platform;
// ... set other fields
Ok(manifest)
```

**`asset_info`** — One-liner after URL building:
```rust
self.authorized_get_json(&url).await
```

**`game_token`** — One-liner:
```rust
self.authorized_get_json(&url).await
```

**`ownership_token`** — Uses POST with form, replace with:
```rust
self.authorized_post_form(&url, &form_params).await
```

**`asset_download_manifests`** — The inner manifest fetch loop: replace `let client = ...` block with:
```rust
match self.get_bytes(&url).await {
    Ok(data) => match DownloadManifest::parse(data.to_vec()) { ... }
    Err(e) => { error!("{:?}", e); }
}
```
Note: The outer loop logic and custom field setting stays — only the HTTP call changes.

**`library_items`** — The inner fetch in the pagination loop: replace with:
```rust
match self.authorized_get_json::<Library>(&url).await {
    Ok(mut records) => { ... pagination logic ... }
    Err(e) => { error!("{:?}", e); break; }
}
```

### 2c. Refactor `src/api/account.rs`

**`account_details`** — Replace with:
```rust
self.authorized_get_json(&url).await
```

**`account_ids_details`** — Build URL with query params, then:
```rust
self.authorized_get_json(parsed_url.as_str()).await
```

**`account_friends`** — One-liner after URL building:
```rust
self.authorized_get_json(&url).await
```

**`user_entitlements`** — One-liner after URL building:
```rust
self.authorized_get_json(&url).await
```

### 2d. Refactor `src/api/fab.rs`

**`fab_asset_manifest`** — This one has special 403 handling. Two options:
- Option A: Don't use the helper, keep custom handling but simplify it
- Option B: Use `authorized_post_json` but handle 403 at the call site

Recommended: Option A — keep the method mostly as-is but use `self.client` (already done in step 1) and simplify the inner match. The 403→FabTimeout mapping is unique enough to warrant its own handling. Just clean up the nested matches and remove redundant `build_client()`.

**`fab_download_manifest`** — Replace inner HTTP call with `self.get_bytes(&point.manifest_url)`:
```rust
match self.get_bytes(&point.manifest_url).await {
    Ok(data) => match DownloadManifest::parse(data.to_vec()) {
        None => Err(EpicAPIError::Unknown),
        Some(man) => Ok(man),
    },
    Err(e) => Err(e),
}
```

**`fab_library_items`** — Pagination loop inner fetch: use `authorized_get_json` similar to `library_items`.

### Verify

```bash
cargo build --lib && cargo test --tests --lib
```

---

## Step 3: Fix `to_vec()` Bugs (Items 3 + 4)

**File:** `src/api/types/download_manifest.rs`

### 3a. Fix `chunk_sha_list.unwrap()` panic (Item 3)

Find in `to_vec()`:
```rust
for sha in self.chunk_sha_list.as_ref().unwrap().values() {
```

Replace with:
```rust
match &self.chunk_sha_list {
    Some(sha_list) => {
        for sha in sha_list.values() {
            match crate::api::utils::decode_hex(sha.as_str()) {
                Ok(mut s) => chunks.append(s.borrow_mut()),
                Err(_) => chunks.append(vec![0u8; 20].borrow_mut()),
            }
        }
    }
    None => {
        for _ in self.chunk_hash_list.keys() {
            chunks.append(vec![0u8; 20].borrow_mut());
        }
    }
}
```

### 3b. Fix `files.resize()` bug (Item 4)

Find in `to_vec()`:
```rust
// flags
// TODO: Figure out what Epic puts in theirs
files.resize(self.file_manifest_list.len(), 0);
```

Replace with:
```rust
// flags — one byte per file (currently zero/unknown)
// TODO: Figure out what Epic puts in theirs
for _ in &self.file_manifest_list {
    files.push(0u8);
}
```

### Verify

```bash
cargo build --lib && cargo test --tests --lib
```

---

## Step 4: Replace Dangerous `unwrap()` Calls (Item 5)

**Files:** Multiple — sweep across the codebase

### 4a. `src/api/login.rs` — refresh_token panic

Find in `start_session`:
```rust
None => [
    ("grant_type".to_string(), "refresh_token".to_string()),
    (
        "refresh_token".to_string(),
        self.user_data.refresh_token.clone().unwrap(),
    ),
    ("token_type".to_string(), "eg1".to_string()),
],
```

Replace with:
```rust
None => {
    let refresh = match &self.user_data.refresh_token {
        Some(t) => t.clone(),
        None => return Err(EpicAPIError::InvalidCredentials),
    };
    [
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh),
        ("token_type".to_string(), "eg1".to_string()),
    ]
}
```

### 4b. `src/api/login.rs` — URL parsing in `invalidate_sesion`

Find:
```rust
match client.delete(Url::from_str(&url).unwrap()).send().await {
```

Replace (now using self.client from step 1):
```rust
let parsed_url = match Url::from_str(&url) {
    Ok(u) => u,
    Err(_) => return false,
};
match self.client.delete(parsed_url).send().await {
```

### 4c. `src/api/egs.rs` — `response.text().await.unwrap()` in warn branches

These are in the non-OK status branches. After step 2, most of these will be inside the helper methods already. But for any remaining (like `asset_download_manifests` loop), replace:
```rust
response.text().await.unwrap()
```
with:
```rust
response.text().await.unwrap_or_default()
```

### 4d. `src/api/fab.rs` — `response.text().await.unwrap()` calls

Same treatment — replace `.unwrap()` with `.unwrap_or_default()` in warn/error logging branches.

### 4e. `src/api/types/download_manifest.rs` — critical panics only

Focus on the most dangerous ones (comprehensive bounds-checking deferred to item 8):

1. **`download_links()` URL parsing:**
   Find: `Url::parse(&format!(...)).unwrap()`
   Replace: Use `Url::parse(...).ok()?` or skip the entry with `continue`

2. **`from_vec()` zlib decompression:**
   Find: `z.read_to_end(&mut data).unwrap();`
   Replace:
   ```rust
   if z.read_to_end(&mut data).is_err() {
       error!("Failed to decompress manifest data");
       return None;
   }
   ```

3. **`to_vec()` chunk GUID encoding:**
   Find: `.collect::<Result<Vec<&str>, _>>().unwrap()` and `u32::from_str_radix(g, 16).unwrap()`
   Replace with graceful fallback — if GUID is malformed, use 0:
   ```rust
   let subs: Vec<&str> = match chunk.as_bytes().chunks(8).map(std::str::from_utf8).collect::<Result<Vec<&str>, _>>() {
       Ok(s) => s,
       Err(_) => { continue; }
   };
   for g in subs {
       chunks.extend_from_slice(&u32::from_str_radix(g, 16).unwrap_or(0).to_le_bytes());
   }
   ```

### Verify

```bash
cargo build --lib && cargo test --tests --lib
```

---

## Final Verification

After all steps:

```bash
cargo build --lib && cargo test --tests --lib
```

Ensure no new warnings (denied in test mode).

## Files Modified Summary

| File | Steps |
|------|-------|
| `src/api/mod.rs` | 1a, 1b, 2a |
| `src/api/egs.rs` | 1c, 2b, 4c |
| `src/api/account.rs` | 2c |
| `src/api/fab.rs` | 1c, 2d, 4d |
| `src/api/login.rs` | 1c, 4a, 4b |
| `src/api/types/download_manifest.rs` | 3a, 3b, 4e |
