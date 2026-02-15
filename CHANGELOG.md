# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-02-15

### Breaking Changes

- **`EpicAPIError::Unknown` removed** — replaced by three typed variants that
  preserve error context:
  - `NetworkError(reqwest::Error)` — transport/connection failures
  - `DeserializationError(String)` — response body parse failures
  - `HttpError { status, body }` — non-success HTTP status codes

  **Migration:** replace any `EpicAPIError::Unknown` match arms with the new
  variants, or use a wildcard `_` arm if you don't need to distinguish them.

- **`DownloadManifest::from_vec` signature changed** — now takes `&[u8]` instead
  of `Vec<u8>`. Pass a slice reference instead of an owned `Vec`.

- **`DownloadManifest::set_custom_field` signature changed** — now takes
  `(&str, &str)` instead of `(String, String)`.

- **`EpicGames::asset_info` signature changed** — now takes `&EpicAsset`
  instead of `EpicAsset`.

- **`EpicGames::ownership_token` signature changed** — now takes `&EpicAsset`
  instead of `EpicAsset`.

- **`UserData::access_token` and `refresh_token` return types changed** —
  now return `Option<&str>` instead of `Option<String>`.

- **`write_fstring` signature changed** — now takes `&str` instead of `String`.

### Added

- `EpicAPIError` now implements `From<reqwest::Error>` for ergonomic `?` usage.
- `try_asset_info`, `try_account_details`, `try_account_friends`,
  `try_user_entitlements`, `try_library_items`, `try_list_assets`,
  `try_game_token` — `Result`-returning facade methods on `EpicGames` that
  expose errors instead of swallowing them.
- `BinaryReader` / `BinaryWriter` — internal bounds-checked binary I/O
  abstraction for manifest parsing.
- 14 new API endpoints: `catalog_items`, `catalog_offers`,
  `bulk_catalog_items`, `currencies`, `library_state_token_status`,
  `service_status`, `offer_prices`, `quick_purchase`, `billing_account`,
  `update_presence`, `external_auths`, `sso_domains`,
  `fab_file_download_info`, `auth_client_credentials`.
- 13 new examples covering all endpoints.
- Comprehensive test suite (10 → 87 tests).
- Doc comments on all public types and API methods.
- Complete README rewrite with auth flow, endpoint tables, and architecture.

### Fixed

- `DownloadManifest::to_vec()` panic when `chunk_sha_list` is `None`.
- `DownloadManifest::to_vec()` buffer truncation caused by `files.resize()`.
- Dangerous `unwrap()` calls replaced with graceful error handling across
  manifest parsing, zlib decompression, and SHA hash conversion.
- HTTP client now reused across requests (was rebuilt per-request, defeating
  connection pooling).

### Changed

- Internal HTTP boilerplate extracted into shared helper methods.
- `download_manifest.rs` decomposed from monolithic parser into named section
  helpers (`parse_header`, `parse_meta`, `parse_chunks`, `parse_files`,
  `parse_custom_fields`).
- Reduced `.clone()` usage across the crate — methods take references where
  possible.

## [0.8.1] - Previous release

See [GitHub releases](https://github.com/AchetaGames/egs-api-rs/releases) for
prior history.
