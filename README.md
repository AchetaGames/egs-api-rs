<p align="center">
<img alt="GitHub" src="https://img.shields.io/github/license/AchetaGames/egs-api-rs">
<a href="https://crates.io/crates/egs-api">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/egs-api"></a>
<a href="https://docs.rs/egs-api/latest/egs_api/">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/egs-api"></a>
<a href="https://discord.gg/C2S8eGfZ6n">
    <img alt="Discord" src="https://img.shields.io/discord/332629362094374913"></a>

</p>

# egs-api

Async Rust client for the Epic Games Store API. Handles authentication, asset
management, download manifest parsing (binary + JSON), and
[Fab](https://www.fab.com/) marketplace integration.

Built on `reqwest` / `tokio`.

## Features

- **Authentication** — OAuth login via authorization code, exchange token, or
  refresh token. Session resume and invalidation.
- **Assets** — List owned assets, fetch catalog metadata (including DLC trees),
  retrieve asset manifests with CDN download URLs.
- **Download Manifests** — Parse Epic's binary and JSON manifest formats.
  Exposes file lists, chunk hashes, and custom fields needed to reconstruct
  downloads.
- **Fab Marketplace** — List Fab library items, fetch Fab asset manifests with
  signed distribution points, and download Fab manifests. Search and browse
  listings with filtering, view listing details, UE format specs, ownership,
  and pricing.
- **Cosmos / Unreal Engine** — Cookie-based session for unrealengine.com.
  EULA management, account details, and engine version blob listing.
- **Account** — Account details, bulk account ID lookup, friends list
  (including pending requests).
- **Entitlements** — Query all user entitlements (games, DLC, subscriptions).
- **Library** — Paginated library listing with optional metadata.
- **Tokens** — Game exchange tokens and per-asset ownership tokens (JWT).
- **Cloud Saves** — List, query, and delete cloud save files.
- **Uplay Integration** — Query and redeem Ubisoft activation codes via
  Epic's GraphQL store API.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
egs-api = "0.9"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use egs_api::EpicGames;

#[tokio::main]
async fn main() {
    let mut egs = EpicGames::new();

    // Authenticate with an authorization code obtained from:
    // https://www.epicgames.com/id/api/redirect?clientId=34a02cf8f4414e29b15921876da36f9a&responseType=code
    let code = "your_authorization_code".to_string();
    if egs.auth_code(None, Some(code)).await {
        println!("Logged in as {}", egs.user_details().display_name.unwrap_or_default());
    }

    // List all owned assets
    let assets = egs.list_assets(None, None).await;
    println!("You own {} assets", assets.len());

    // Get detailed info for the first asset
    if let Some(asset) = assets.first() {
        if let Some(info) = egs.asset_info(asset).await {
            println!("{}: {}", info.id, info.title.unwrap_or_default());
        }
    }
}
```

## Authentication Flow

Epic uses OAuth2 with a launcher client ID. The typical flow is:

1. Open the authorization URL in a browser — the user logs in and is redirected
   to a JSON page containing an `authorizationCode`.
2. Pass that code to `egs.auth_code(None, Some(code))`.
3. On success, `egs.user_details()` contains the session tokens.

To persist the session across runs, serialize `egs.user_details()` (it
implements `Serialize` / `Deserialize`) and restore it later with
`egs.set_user_details(saved)` followed by `egs.login()` which will use the
refresh token to re-authenticate.

**Authorization URL:**
```
https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode
```

See the [`auth`](examples/auth.rs) example for a complete interactive flow with
token persistence.

## Examples

The crate ships with examples covering every endpoint. Run them with:

```bash
# First: authenticate and save a token
cargo run --example auth

# Then run any of these (they reuse the saved token):
cargo run --example account              # Account details, ID lookup, friends, external auths, SSO
cargo run --example entitlements         # List all entitlements
cargo run --example library              # Paginated library listing
cargo run --example assets               # Full pipeline: list → info → manifest → download manifest
cargo run --example game_token           # Exchange code + ownership token
cargo run --example fab                  # Fab search, browse, listing detail, library, downloads
cargo run --example cosmos               # Cosmos session, EULA, account, engine versions
cargo run --example catalog              # Catalog items, offers, bulk lookup
cargo run --example commerce             # Currencies, prices, billing, quick purchase
cargo run --example status               # Service status (lightswitch API)
cargo run --example presence             # Update online presence
cargo run --example client_credentials   # App-level auth + library state tokens
cargo run --example cloud_saves          # Cloud save file listing + management
cargo run --example uplay                # Ubisoft activation code queries
```

## API Overview

The public API is the [`EpicGames`](https://docs.rs/egs-api/latest/egs_api/struct.EpicGames.html)
struct. It wraps an internal HTTP client with cookie storage and bearer token
management. Most methods return `Option<T>` or `Vec<T>` (swallowing transport
errors); Fab methods return `Result<T, EpicAPIError>` to expose timeout/error
distinctions.

### Authentication

| Method | Description |
|--------|-------------|
| `auth_code(exchange_token, authorization_code)` | Start a new session |
| `auth_sid(sid)` | Authenticate via SID cookie (web-based exchange code flow) |
| `auth_client_credentials()` | App-level auth (no user context) |
| `login()` | Resume session using saved refresh token |
| `logout()` | Invalidate current session |
| `is_logged_in()` | Check if access token is still valid (>600s remaining) |
| `user_details()` / `set_user_details(data)` | Get/set session state for persistence |

### Epic Games Store

| Method | Description |
|--------|-------------|
| `list_assets(platform, label)` | List all owned assets (default: Windows/Live) |
| `asset_info(asset)` | Catalog metadata for an asset (includes DLC list) |
| `asset_manifest(platform, label, namespace, item_id, app)` | CDN manifest with download URLs |
| `asset_download_manifests(manifest)` | Parse binary/JSON download manifests from all CDN mirrors |
| `catalog_items(namespace, start, count)` | Paginated catalog items for a namespace |
| `catalog_offers(namespace, start, count)` | Paginated catalog offers for a namespace |
| `bulk_catalog_items(items)` | Bulk fetch catalog items across namespaces |
| `currencies(start, count)` | Available currencies with symbols and decimals |
| `game_token()` | Short-lived exchange code for game launches |
| `ownership_token(asset)` | JWT proving asset ownership |
| `library_state_token_status(token_id)` | Check library state token validity |
| `artifact_service_ticket(platform, label, namespace, item_id, app)` | Artifact service download ticket |
| `game_manifest_by_ticket(platform, label, namespace, item_id, app, ticket)` | Game manifest via artifact service ticket |
| `launcher_manifests(platform, label, namespace, item_id, app)` | Launcher asset manifests |
| `delta_manifest(manifest, old_build_id, new_build_id)` | Delta/patch manifest between two builds |

### Cloud Saves

| Method | Description |
|--------|-------------|
| `cloud_save_list()` | List all cloud save files for the current user |
| `cloud_save_query(filename)` | Query metadata for a specific cloud save file |
| `cloud_save_delete(filename)` | Delete a cloud save file |

### Uplay / Store Integration

| Method | Description |
|--------|-------------|
| `store_get_uplay_codes(namespace, offer_id)` | Query Ubisoft activation codes for a game |
| `store_claim_uplay_code(namespace, offer_id)` | Claim a Ubisoft activation code |
| `store_redeem_uplay_codes(uplay_codes)` | Redeem Ubisoft activation codes |

### Fab Marketplace

| Method | Description |
|--------|-------------|
| `fab_library_items(account_id)` | List all Fab library items (paginated) |
| `fab_asset_manifest(artifact_id, namespace, asset_id, platform)` | Signed download info with distribution points |
| `fab_download_manifest(download_info, distribution_point_url)` | Parse download manifest from a specific CDN |
| `fab_file_download_info(listing_id, format_id, file_id)` | Download info for a specific Fab file |
| `fab_search(params)` | Search/browse listings with filters, sorting, pagination |
| `fab_listing(uid)` | Full listing detail (title, seller, category, ratings) |
| `fab_listing_ue_formats(uid)` | UE-specific format specs (engine versions, platforms) |
| `fab_listing_state(uid)` | Ownership, wishlist, and review state for a listing |
| `fab_listing_states_bulk(listing_ids)` | Bulk check listing states |
| `fab_bulk_prices(offer_ids)` | Bulk fetch pricing for multiple offers |
| `fab_listing_ownership(uid)` | Detailed ownership/license info for a listing |

### Cosmos / Unreal Engine

| Method | Description |
|--------|-------------|
| `cosmos_session_setup(exchange_code)` | Set up a Cosmos cookie session from an exchange code |
| `cosmos_auth_upgrade()` | Upgrade bearer token to Cosmos session |
| `cosmos_eula_check(eula_id, locale)` | Check if a EULA has been accepted |
| `cosmos_eula_accept(eula_id, locale, version)` | Accept a EULA |
| `cosmos_account()` | Cosmos account details |
| `cosmos_policy_aodc()` | Age of Digital Consent policy status |
| `cosmos_comm_opt_in(setting)` | Communication opt-in status |
| `engine_versions(platform)` | Engine version download blobs for a platform |

### Account & Social

| Method | Description |
|--------|-------------|
| `account_details()` | Email, display name, country, 2FA status |
| `account_ids_details(ids)` | Bulk lookup of account IDs to display names |
| `account_friends(include_pending)` | Friends list with pending request status |
| `external_auths(account_id)` | Linked platform accounts (Steam, PSN, Xbox, etc.) |
| `sso_domains()` | SSO domain list for cookie sharing |
| `user_entitlements()` | All entitlements (games, DLC, subscriptions) |
| `library_items(include_metadata)` | Library records with optional metadata |

### Commerce

| Method | Description |
|--------|-------------|
| `offer_prices(namespace, offer_ids, country)` | Offer pricing with formatted strings |
| `quick_purchase(namespace, offer_id)` | Quick purchase (free game claims) |
| `billing_account()` | Default billing account and country |

### Status & Presence

| Method | Description |
|--------|-------------|
| `service_status(service_id)` | Service operational status (lightswitch API) |
| `update_presence(session_id, body)` | Update user online presence |

## Download Manifest Format

The [`DownloadManifest`](https://docs.rs/egs-api/latest/egs_api/api/types/download_manifest/struct.DownloadManifest.html)
struct handles Epic's binary manifest format (with JSON fallback). The binary
format is little-endian, optionally zlib-compressed:

```
Header (41 bytes):
  magic(u32) → header_size(u32) → size_uncompressed(u32) →
  size_compressed(u32) → sha_hash(20 bytes) → compressed(u8) → version(u32)

Body (zlib-compressed):
  Meta:   feature_level, is_file_data, app_id, app_name, build_version, launch_exe, ...
  Chunks: guid(16 bytes) × N, hash(u64) × N, sha(20 bytes) × N, group(u8) × N, ...
  Files:  filename × N, symlink_target × N, sha_hash(20 bytes) × N, install_tags × N, chunk_parts × N
  Custom: key-value pairs (e.g., BaseUrl, CatalogItemId, BuildLabel)
```

Use `DownloadManifest::parse(data)` to parse either format. Access file lists
via `file_manifest_list`, chunk info via `chunk_hash_list`, and custom fields
via `custom_field(key)`.

## Architecture

```
EpicGames (public facade, src/lib.rs)
  └── EpicAPI (internal, src/api/mod.rs)
        ├── login.rs    — OAuth: start, resume, invalidate
        ├── egs.rs      — Assets, manifests, library, cloud saves, tokens
        ├── account.rs  — Account details, friends, entitlements
        ├── fab.rs      — Fab marketplace: library, downloads, search, browse
        ├── cosmos.rs   — Cosmos session, EULA, account, engine versions
        └── store.rs    — Uplay/Ubisoft code redemption (GraphQL)
```

`EpicGames` is the consumer-facing struct. It delegates to `EpicAPI` which
holds the `reqwest::Client` (with cookie store) and `UserData` (session state).
API methods are split across files via `impl EpicAPI` blocks.

## License

MIT
