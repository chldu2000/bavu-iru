# Phase 4: Import/Export + Data Integrity — Design Spec

## Overview

Add import/export functionality supporting 5 external formats + 1 encrypted backup format, plus a standalone data integrity check tool. All logic lives in a single `import_export.rs` module, with 3 new frontend components.

## Decisions

- **Architecture**: Single-module centralized (方案 A) — all format parsing in `import_export.rs`
- **Conflict handling**: Default to keeping both entries (suffix-marked), user decides per-entry in preview
- **Export formats**: Support both plaintext (CSV/JSON) and encrypted backup (`.bvault` with independent password)

## Backend: 4 New Tauri Commands

### `export_vault(format, password?)`

- `format`: `"csv"` | `"json"` | `"encrypted"`
- `password`: required when format is `"encrypted"` (export password, independent from master)
- Requires master password re-verification via `keyring::verify_password()`
- Returns `{ filename: string, data: string }` (data is base64 for encrypted, utf8 for csv/json)
- Reuses `cipher.rs` AES-256-GCM + `kdf.rs` Argon2id for encrypted export

### `preview_import(format, content)`

- `format`: `"csv"` | `"json"` | `"bitwarden"` | `"keepass"` | `"chrome"` | `"encrypted"`
- `content`: file content as string (for encrypted, includes base64 ciphertext)
- For encrypted format, prompts for export password to decrypt first
- Does NOT write to database
- Returns `{ total: number, entries: Entry[], duplicates: Duplicate[] }`
- Duplicate detection: match by `title + username`

### `import_vault(format, content, resolutions)`

- `format`: same as preview_import
- `content`: file content
- `resolutions`: `Map<string, "keep" | "skip" | "replace">` — per-duplicate decision
- Uses existing `repository::create_entry()` for consistent encryption
- Returns `{ imported: number, skipped: number, replaced: number }`

### `check_integrity()`

- No parameters (uses current vault state)
- Returns `{ status: "ok" | "warning" | "error", issues: IntegrityIssue[] }`
- `IntegrityIssue = { severity: "warning" | "error", message: string }`

## Format Mapping

Each format has a `parse → Vec<Entry>` function:

| Format | Source Fields | Target Entry Fields |
|--------|--------------|---------------------|
| Native JSON | Direct serialization | All fields (lossless) |
| Generic CSV | title, username, password, url, notes, folder | title, username, password, url, notes, folder_id |
| Bitwarden JSON | items[].login.{username, password, uris[]}, notes, fields[] | title, username, password, url, notes, custom_fields |
| KeePass CSV | Account, Login Name, Password, Web Site, Comments, Group | title, username, password, url, notes, folder_id |
| Chrome/Firefox CSV | name, url, username, password, note | title, username, password, url, notes |

## Encrypted Backup Format (`.bvault`)

Binary structure:

```
Header:
  magic:     b"BVLT"        (4 bytes)
  version:   u16 = 1        (2 bytes)
  salt_len:  u16            (2 bytes)
  salt:      [u8; 32]       (32 bytes)
  nonce:     [u8; 12]       (12 bytes)
Data:
  ciphertext: AES-256-GCM encrypted JSON  (variable length)
Footer:
  auth_tag:  [u8; 16]       (16 bytes)
```

- Export password derives independent key via Argon2id (same params as vault KDF, separate salt)
- JSON payload contains `{ entries, folders, tags }` — full vault snapshot
- Import decrypts with export password, then flows through normal `preview_import` → `import_vault`

## Data Integrity Check

Checks performed by `check_integrity`:

| Check | Method | Severity on Failure |
|-------|--------|-------------------|
| Database readable | Open SQLite, run simple query | error |
| Metadata complete | Verify `kdf_salt`, `vault_verify` meta keys exist | error |
| Entries decryptable | Decrypt `title` field per entry, verify AES-GCM auth tag | error |
| Orphan entries | Check `folder_id` references exist in folders table | warning |
| Orphan tag relations | Check `entry_tags` references valid entry_id and tag_id | warning |

## Frontend: New Components

### `ImportExport.svelte`

Main interface with 3 tabs: Import, Export, Integrity Check. Accessed via sidebar button alongside Settings.

### `ImportPreview.svelte`

Table showing parsed entries. Duplicate entries highlighted with per-row dropdown: Keep Both / Skip / Replace (default: Keep Both). Shows total count, duplicate count, and confirm button.

### `ExportConfirm.svelte`

Modal dialog. Format selector (CSV / JSON / Encrypted). Master password input for verification. When encrypted: additional export password + confirm field. Export button triggers download.

## Integration Points

- **Settings view**: Add "Import / Export" entry in sidebar, parallel to existing Settings button
- **tauri.ts**: Add `invoke` wrappers for 4 new commands
- **entries store**: No changes needed — import uses existing `load()` to refresh after import
- **Error handling**: Use existing `error.rs` types, add `ImportError` and `ExportError` variants
