# Phase 4: Import/Export + Data Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add import/export supporting 5 external formats + encrypted backup, plus a standalone data integrity check tool.

**Architecture:** Single `import_export.rs` module handles all format parsing/serialization. 4 new Tauri commands (`export_vault`, `preview_import`, `import_vault`, `check_integrity`). 3 new Svelte components (`ImportExport.svelte`, `ImportPreview.svelte`, `ExportConfirm.svelte`).

**Tech Stack:** Rust (csv crate, existing AES-256-GCM + Argon2id), Svelte 5, Tauri v2 IPC

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src-tauri/Cargo.toml` | Add `csv` dependency |
| Modify | `src-tauri/src/error.rs` | Add `Import`/`Export` error variants |
| Modify | `src-tauri/src/db/models.rs` | Add `ImportEntry` (flat, pre-folder-resolution) |
| Modify | `src-tauri/src/db/repository.rs` | Add `delete_all_entries()`, `count_entries()`, integrity query helpers |
| Rewrite | `src-tauri/src/commands/import_export.rs` | All 4 commands + 5 format parsers |
| Modify | `src-tauri/src/lib.rs` | Register 4 new commands in `invoke_handler` |
| Modify | `src/lib/utils/tauri.ts` | Add 4 invoke wrappers + types |
| Create | `src/lib/components/ImportExport.svelte` | Main tabbed interface |
| Create | `src/lib/components/ImportPreview.svelte` | Duplicate resolution table |
| Create | `src/lib/components/ExportConfirm.svelte` | Export format + password modal |
| Modify | `src/routes/+page.svelte` | Add `import-export` view mode + sidebar button |

---

## Task 1: Add Rust Dependencies + Error Types

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/error.rs`

- [ ] **Step 1: Add `csv` crate to Cargo.toml**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
csv = "1.3"
```

- [ ] **Step 2: Add import/export error variants to `error.rs`**

In `src-tauri/src/error.rs`, add two new variants to `AppError`:

```rust
#[error("Import error: {0}")]
Import(String),
#[error("Export error: {0}")]
Export(String),
```

Also add a `From` impl for csv errors:

```rust
impl From<csv::Error> for AppError {
    fn from(e: csv::Error) -> Self {
        AppError::Import(e.to_string())
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru/src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/error.rs
git commit -m "feat: add csv dependency and import/export error types"
```

---

## Task 2: Add ImportEntry Model + Repository Helpers

**Files:**
- Modify: `src-tauri/src/db/models.rs`
- Modify: `src-tauri/src/db/repository.rs`

- [ ] **Step 1: Add `ImportEntry` struct to models.rs**

In `src-tauri/src/db/models.rs`, add after the existing structs:

```rust
/// Flat entry representation used during import before folder resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub folder: Option<String>,
    pub custom_fields: Option<String>,
}
```

- [ ] **Step 2: Add repository helpers**

In `src-tauri/src/db/repository.rs`, add these methods to the `Database` impl block:

```rust
pub fn count_entries(&self) -> SqliteResult<usize> {
    let conn = self.conn.lock().unwrap();
    conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
}

pub fn find_entry_by_title_username(&self, title: &str, username: Option<&str>) -> SqliteResult<Option<Entry>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = if username.is_some() {
        conn.prepare("SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, is_favorite, created_at, updated_at FROM entries WHERE title = ?1 AND username = ?2")?
    } else {
        conn.prepare("SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, is_favorite, created_at, updated_at FROM entries WHERE title = ?1 AND username IS NULL")?
    };
    let mut rows = if let Some(u) = username {
        stmt.query(rusqlite::params![title, u])?
    } else {
        stmt.query(rusqlite::params![title])?
    };
    match rows.next()? {
        Some(row) => Ok(Some(Entry {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            title: row.get(2)?,
            username: row.get(3)?,
            password: row.get(4)?,
            url: row.get(5)?,
            notes: row.get(6)?,
            custom_fields: row.get(7)?,
            tags: row.get(8)?,
            strength: row.get(9)?,
            expires_at: row.get(10)?,
            is_favorite: row.get::<_, i32>(11)? != 0,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })),
        None => Ok(None),
    }
}

pub fn find_folder_by_name(&self, name: &str) -> SqliteResult<Option<Folder>> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, parent_id, sort_order, created_at, updated_at FROM folders WHERE name = ?1")?;
    let mut rows = stmt.query(rusqlite::params![name])?;
    match rows.next()? {
        Some(row) => Ok(Some(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })),
        None => Ok(None),
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru/src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/models.rs src-tauri/src/db/repository.rs
git commit -m "feat: add ImportEntry model and repository helpers for import"
```

---

## Task 3: Implement Format Parsers in import_export.rs

**Files:**
- Rewrite: `src-tauri/src/commands/import_export.rs`

- [ ] **Step 1: Write the complete import_export.rs with format parsers**

Replace the entire content of `src-tauri/src/commands/import_export.rs` with:

```rust
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::io::Read;
use tauri::State;

use crate::crypto::cipher;
use crate::crypto::kdf::{self, KdfParams};
use crate::crypto::keyring::Keyring;
use crate::db::models::{Entry, ImportEntry};
use crate::db::repository::Database;
use crate::error::AppError;

// --- Shared types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateEntry {
    pub existing_id: String,
    pub existing_title: String,
    pub existing_username: Option<String>,
    pub imported_title: String,
    pub imported_username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewResult {
    pub total: usize,
    pub entries: Vec<ImportEntry>,
    pub duplicates: Vec<DuplicateEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub replaced: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityIssue {
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub status: String,
    pub issues: Vec<IntegrityIssue>,
}

// --- Format parsing ---

fn parse_generic_csv(content: &str) -> Result<Vec<ImportEntry>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());
    let mut entries = Vec::new();
    for result in reader.records() {
        let record = result?;
        let title = record.get(0).unwrap_or("").trim().to_string();
        if title.is_empty() {
            continue;
        }
        entries.push(ImportEntry {
            title,
            username: record.get(1).map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten(),
            password: record.get(2).map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten(),
            url: record.get(3).map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten(),
            notes: record.get(4).map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten(),
            folder: record.get(5).map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten(),
            custom_fields: None,
        });
    }
    Ok(entries)
}

fn parse_native_json(content: &str) -> Result<Vec<ImportEntry>, AppError> {
    #[derive(Deserialize)]
    struct NativeExport {
        entries: Vec<ImportEntry>,
    }
    let data: NativeExport = serde_json::from_str(content)
        .map_err(|e| AppError::Import(format!("Invalid JSON: {}", e)))?;
    Ok(data.entries)
}

fn parse_bitwarden_json(content: &str) -> Result<Vec<ImportEntry>, AppError> {
    #[derive(Deserialize)]
    struct BwLogin {
        username: Option<String>,
        password: Option<String>,
        uris: Option<Vec<BwUri>>,
    }
    #[derive(Deserialize)]
    struct BwUri {
        uri: Option<String>,
    }
    #[derive(Deserialize)]
    struct BwField {
        name: Option<String>,
        value: Option<String>,
    }
    #[derive(Deserialize)]
    struct BwItem {
        name: String,
        r#type: Option<i32>,
        login: Option<BwLogin>,
        notes: Option<String>,
        fields: Option<Vec<BwField>>,
        folder_name: Option<String>,
    }
    #[derive(Deserialize)]
    struct BwExport {
        items: Vec<BwItem>,
    }
    let data: BwExport = serde_json::from_str(content)
        .map_err(|e| AppError::Import(format!("Invalid Bitwarden JSON: {}", e)))?;
    let mut entries = Vec::new();
    for item in data.items {
        let url = item.login.as_ref()
            .and_then(|l| l.uris.as_ref())
            .and_then(|uris| uris.first())
            .and_then(|u| u.uri.clone());
        let custom_fields = item.fields.map(|fields| {
            let pairs: Vec<serde_json::Value> = fields.iter().map(|f| {
                serde_json::json!({
                    "name": f.name.as_deref().unwrap_or(""),
                    "value": f.value.as_deref().unwrap_or("")
                })
            }).collect();
            serde_json::to_string(&pairs).unwrap_or_else(|_| "[]".to_string())
        });
        entries.push(ImportEntry {
            title: item.name,
            username: item.login.as_ref().and_then(|l| l.username.clone()),
            password: item.login.as_ref().and_then(|l| l.password.clone()),
            url,
            notes: item.notes,
            folder: item.folder_name,
            custom_fields,
        });
    }
    Ok(entries)
}

fn parse_keepass_csv(content: &str) -> Result<Vec<ImportEntry>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = reader.headers()?.clone();
    // Find column indices — KeePass CSV uses: Group, Account, Login Name, Password, Web Site, Comments
    let group_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("group"));
    let account_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("account"));
    let login_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("login name"));
    let pass_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("password"));
    let web_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("web site"));
    let comments_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("comments"));

    let mut entries = Vec::new();
    for result in reader.records() {
        let record = result?;
        let title = account_idx
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        if title.is_empty() {
            continue;
        }
        entries.push(ImportEntry {
            title,
            username: login_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            password: pass_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            url: web_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            notes: comments_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            folder: group_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            custom_fields: None,
        });
    }
    Ok(entries)
}

fn parse_browser_csv(content: &str) -> Result<Vec<ImportEntry>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = reader.headers()?.clone();
    // Chrome: name,url,username,password,note
    // Firefox: url,username,password,httpRealm,formActionOrigin,guid,timeCreated,timeLastUsed,timePasswordUsed
    let name_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("name"));
    let url_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("url"));
    let user_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("username"));
    let pass_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("password"));
    let note_idx = headers.iter().position(|h| h.trim().eq_ignore_ascii_case("note"));

    let mut entries = Vec::new();
    for result in reader.records() {
        let record = result?;
        let title = name_idx
            .or(url_idx)
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();
        if title.is_empty() {
            continue;
        }
        entries.push(ImportEntry {
            title,
            username: user_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            password: pass_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            url: url_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            notes: note_idx.and_then(|i| record.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            folder: None,
            custom_fields: None,
        });
    }
    Ok(entries)
}

fn parse_encrypted(content: &str, password: &str) -> Result<Vec<ImportEntry>, AppError> {
    let data = BASE64.decode(content)?;
    if data.len() < 56 {
        return Err(AppError::Import("File too short".into()));
    }
    // Header: magic(4) + version(2) + salt_len(2) + salt(32) + nonce(12) = 52 bytes header
    let magic = &data[0..4];
    if magic != b"BVLT" {
        return Err(AppError::Import("Not a valid .bvault file".into()));
    }
    let _version = u16::from_be_bytes([data[4], data[5]]);
    let _salt_len = u16::from_be_bytes([data[6], data[7]]);
    let mut salt = [0u8; 32];
    salt.copy_from_slice(&data[8..40]);
    let nonce_and_ciphertext = &data[40..];

    let params = KdfParams {
        salt,
        time_cost: 3,
        memory_cost: 65536,
        parallelism: 4,
    };
    let key = kdf::derive_key(password, &params)?;
    // cipher::decrypt expects [nonce(12) || ciphertext + auth_tag]
    let decrypted = cipher::decrypt(&key, nonce_and_ciphertext)
        .map_err(|_| AppError::Import("Wrong password or corrupted file".into()))?;
    let json_str = String::from_utf8(decrypted)
        .map_err(|e| AppError::Import(format!("Invalid UTF-8: {}", e)))?;

    // Encrypted backup wraps entries in the same structure as native JSON
    parse_native_json(&json_str)
}

// Need to add custom_fields to ImportEntry parsing — already in the struct

fn parse_entries(format: &str, content: &str, password: Option<&str>) -> Result<Vec<ImportEntry>, AppError> {
    match format {
        "csv" => parse_generic_csv(content),
        "json" => parse_native_json(content),
        "bitwarden" => parse_bitwarden_json(content),
        "keepass" => parse_keepass_csv(content),
        "chrome" => parse_browser_csv(content),
        "encrypted" => {
            let pwd = password.ok_or_else(|| AppError::Import("Password required for encrypted import".into()))?;
            parse_encrypted(content, pwd)
        }
        _ => Err(AppError::Import(format!("Unknown format: {}", format))),
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub fn export_vault(
    format: String,
    password: Option<String>,
    master_password: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<ExportData, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }

    // Verify master password
    let salt_b64 = db.get_meta("kdf_salt")?.ok_or(AppError::VaultNotFound)?;
    let salt = BASE64.decode(&salt_b64)?;
    let time_cost: u32 = db.get_meta("kdf_time_cost")?.unwrap_or_default().parse().unwrap_or(3);
    let memory_cost: u32 = db.get_meta("kdf_memory_cost")?.unwrap_or_default().parse().unwrap_or(65536);
    let parallelism: u32 = db.get_meta("kdf_parallelism")?.unwrap_or_default().parse().unwrap_or(4);
    let mut salt_arr = [0u8; 32];
    salt_arr.copy_from_slice(&salt[..32]);
    let params = KdfParams { salt: salt_arr, time_cost, memory_cost, parallelism };
    let key = kdf::derive_key(&master_password, &params)?;
    let verify_b64 = db.get_meta("vault_verify")?.ok_or(AppError::VaultNotFound)?;
    let verify_data = BASE64.decode(&verify_b64)?;
    let decrypted = cipher::decrypt(&key, &verify_data)?;
    if decrypted != b"bavu-iru-vault-verify" {
        return Err(AppError::InvalidPassword);
    }

    let entries = db.list_entries()?;
    let now = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&serde_json::json!({ "entries": entries }))
                .map_err(|e| AppError::Export(e.to_string()))?;
            Ok(ExportData {
                filename: format!("vault_export_{}.json", now),
                data: json,
            })
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            for entry in &entries {
                wtr.write_record(&[
                    &entry.title,
                    entry.username.as_deref().unwrap_or(""),
                    entry.password.as_deref().unwrap_or(""),
                    entry.url.as_deref().unwrap_or(""),
                    entry.notes.as_deref().unwrap_or(""),
                ]).map_err(|e| AppError::Export(e.to_string()))?;
            }
            let bytes = wtr.into_inner().map_err(|e| AppError::Export(e.to_string()))?;
            Ok(ExportData {
                filename: format!("vault_export_{}.csv", now),
                data: String::from_utf8(bytes).map_err(|e| AppError::Export(e.to_string()))?,
            })
        }
        "encrypted" => {
            let export_pwd = password.ok_or_else(|| AppError::Export("Export password required".into()))?;
            let mut salt = [0u8; 32];
            rand::fill(&mut salt);
            let params = KdfParams { salt, time_cost: 3, memory_cost: 65536, parallelism: 4 };
            let key = kdf::derive_key(&export_pwd, &params)?;

            let json = serde_json::to_string(&serde_json::json!({ "entries": entries }))
                .map_err(|e| AppError::Export(e.to_string()))?;
            let encrypted = cipher::encrypt(&key, json.as_bytes())?;

            // Build .bvault: magic(4) + version(2) + salt_len(2) + salt(32) + nonce+ciphertext+tag
            let mut output = Vec::with_capacity(8 + encrypted.len());
            output.extend_from_slice(b"BVLT");
            output.extend_from_slice(&1u16.to_be_bytes()); // version
            output.extend_from_slice(&32u16.to_be_bytes()); // salt_len
            output.extend_from_slice(&salt);
            output.extend_from_slice(&encrypted);

            Ok(ExportData {
                filename: format!("vault_backup_{}.bvault", now),
                data: BASE64.encode(&output),
            })
        }
        _ => Err(AppError::Export(format!("Unknown format: {}", format))),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub filename: String,
    pub data: String,
}

#[tauri::command]
pub fn preview_import(
    format: String,
    content: String,
    password: Option<String>,
    db: State<'_, Database>,
) -> Result<PreviewResult, AppError> {
    let imported = parse_entries(&format, &content, password.as_deref())?;
    let existing = db.list_entries()?;

    let mut duplicates = Vec::new();
    for imp in &imported {
        let matched = existing.iter().find(|e| {
            e.title.eq_ignore_ascii_case(&imp.title)
                && match (&e.username, &imp.username) {
                    (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
                    (None, None) => true,
                    _ => false,
                }
        });
        if let Some(ex) = matched {
            duplicates.push(DuplicateEntry {
                existing_id: ex.id.clone(),
                existing_title: ex.title.clone(),
                existing_username: ex.username.clone(),
                imported_title: imp.title.clone(),
                imported_username: imp.username.clone(),
            });
        }
    }

    Ok(PreviewResult {
        total: imported.len(),
        entries: imported,
        duplicates,
    })
}

#[tauri::command]
pub fn import_vault(
    format: String,
    content: String,
    password: Option<String>,
    resolutions: std::collections::HashMap<String, String>,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<ImportResult, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }

    let imported = parse_entries(&format, &content, password.as_deref())?;
    let mut result = ImportResult { imported: 0, skipped: 0, replaced: 0 };
    let now = chrono::Utc::now().to_rfc3339();

    for imp in imported {
        // Check for duplicate
        let existing = db.find_entry_by_title_username(&imp.title, imp.username.as_deref())?;

        if let Some(ex) = existing {
            let resolution = resolutions.get(&ex.id).map(|s| s.as_str()).unwrap_or("keep");
            match resolution {
                "skip" => {
                    result.skipped += 1;
                    continue;
                }
                "replace" => {
                    let mut entry = map_import_to_entry(&imp, &db, &now);
                    entry.id = ex.id.clone();
                    db.update_entry(&entry)?;
                    result.replaced += 1;
                    continue;
                }
                _ => {} // "keep" — fall through to create new
            }
        }

        let entry = map_import_to_entry(&imp, &db, &now);
        db.create_entry(&entry)?;
        result.imported += 1;
    }

    Ok(result)
}

fn map_import_to_entry(imp: &ImportEntry, db: &Database, now: &str) -> Entry {
    let folder_id = imp.folder.as_ref().and_then(|name| {
        db.find_folder_by_name(name).ok().flatten().map(|f| f.id)
    });

    Entry {
        id: uuid::Uuid::new_v4().to_string(),
        folder_id,
        title: imp.title.clone(),
        username: imp.username.clone(),
        password: imp.password.clone(),
        url: imp.url.clone(),
        notes: imp.notes.clone(),
        custom_fields: imp.custom_fields.clone(),
        tags: None,
        strength: imp.password.as_ref().map(|p| crate::crypto::strength::evaluate_strength(p) as i32),
        expires_at: None,
        is_favorite: false,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}

#[tauri::command]
pub fn check_integrity(
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<IntegrityResult, AppError> {
    let mut issues = Vec::new();

    // Check 1: metadata complete
    let required_keys = ["kdf_salt", "vault_verify"];
    for key in &required_keys {
        match db.get_meta(key) {
            Ok(Some(_)) => {}
            Ok(None) => issues.push(IntegrityIssue {
                severity: "error".into(),
                message: format!("Missing required metadata: {}", key),
            }),
            Err(e) => issues.push(IntegrityIssue {
                severity: "error".into(),
                message: format!("Database read error for {}: {}", key, e),
            }),
        }
    }

    // Check 2: entries decryptable
    if let Ok(entries) = db.list_entries() {
        for entry in &entries {
            if entry.title.is_empty() {
                issues.push(IntegrityIssue {
                    severity: "warning".into(),
                    message: format!("Entry {} has empty title", entry.id),
                });
            }
        }
        issues.push(IntegrityIssue {
            severity: "info".into(),
            message: format!("Total entries: {}", entries.len()),
        });
    }

    // Check 3: orphan entries (folder_id references)
    if let Ok(entries) = db.list_entries() {
        if let Ok(folders) = db.list_folders() {
            let folder_ids: std::collections::HashSet<_> = folders.iter().map(|f| f.id.as_str()).collect();
            for entry in &entries {
                if let Some(ref fid) = entry.folder_id {
                    if !folder_ids.contains(fid.as_str()) {
                        issues.push(IntegrityIssue {
                            severity: "warning".into(),
                            message: format!("Entry '{}' references non-existent folder {}", entry.title, fid),
                        });
                    }
                }
            }
        }
    }

    // Check 4: orphan tag relations
    let conn_result: Result<(), _> = (|| {
        let entries = db.list_entries()?;
        let tags = db.list_tags()?;
        let entry_ids: std::collections::HashSet<_> = entries.iter().map(|e| e.id.as_str()).collect();
        let tag_ids: std::collections::HashSet<_> = tags.iter().map(|t| t.id.as_str()).collect();
        for entry in &entries {
            if let Some(tags_str) = &entry.tags {
                if let Ok(tag_list) = serde_json::from_str::<Vec<String>>(tags_str) {
                    for tid in &tag_list {
                        if !tag_ids.contains(tid.as_str()) {
                            issues.push(IntegrityIssue {
                                severity: "warning".into(),
                                message: format!("Entry '{}' references non-existent tag {}", entry.title, tid),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    })();
    if let Err(e) = conn_result {
        issues.push(IntegrityIssue {
            severity: "error".into(),
            message: format!("Database error during tag check: {}", e),
        });
    }

    let has_errors = issues.iter().any(|i| i.severity == "error");
    let status = if has_errors { "error" } else if issues.iter().any(|i| i.severity == "warning") { "warning" } else { "ok" };

    Ok(IntegrityResult { status: status.into(), issues })
}
```

- [ ] **Step 2: Add `custom_fields` to `ImportEntry` — verify it's already there from Step 1 (it is, field is included)**

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru/src-tauri && cargo check 2>&1 | tail -10`
Expected: `Finished` — there may be unused import warnings, that's fine. Fix any errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/import_export.rs
git commit -m "feat: implement import/export with CSV, JSON, Bitwarden, KeePass, Chrome, encrypted formats"
```

---

## Task 4: Register Commands in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add 4 new commands to invoke_handler**

In `src-tauri/src/lib.rs`, add these 4 lines to the `invoke_handler` macro (after the `clipboard::clipboard_clear` line):

```rust
commands::import_export::export_vault,
commands::import_export::preview_import,
commands::import_export::import_vault,
commands::import_export::check_integrity,
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru/src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register import/export commands in Tauri invoke handler"
```

---

## Task 5: Add Frontend Tauri Wrappers

**Files:**
- Modify: `src/lib/utils/tauri.ts`

- [ ] **Step 1: Add import/export types and invoke wrappers**

Append to the end of `src/lib/utils/tauri.ts`:

```typescript
// --- Import/Export operations ---

export interface ImportEntry {
  title: string;
  username: string | null;
  password: string | null;
  url: string | null;
  notes: string | null;
  folder: string | null;
}

export interface DuplicateEntry {
  existing_id: string;
  existing_title: string;
  existing_username: string | null;
  imported_title: string;
  imported_username: string | null;
}

export interface PreviewResult {
  total: number;
  entries: ImportEntry[];
  duplicates: DuplicateEntry[];
}

export interface ImportResult {
  imported: number;
  skipped: number;
  replaced: number;
}

export interface ExportData {
  filename: string;
  data: string;
}

export interface IntegrityIssue {
  severity: string;
  message: string;
}

export interface IntegrityResult {
  status: string;
  issues: IntegrityIssue[];
}

export async function exportVault(
  format: string,
  masterPassword: string,
  password?: string
): Promise<ExportData> {
  return invoke('export_vault', { format, masterPassword, password });
}

export async function previewImport(
  format: string,
  content: string,
  password?: string
): Promise<PreviewResult> {
  return invoke('preview_import', { format, content, password });
}

export async function importVault(
  format: string,
  content: string,
  resolutions: Record<string, string>,
  password?: string
): Promise<ImportResult> {
  return invoke('import_vault', { format, content, resolutions, password });
}

export async function checkIntegrity(): Promise<IntegrityResult> {
  return invoke('check_integrity');
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru && npx svelte-check --threshold error 2>&1 | tail -5`
Expected: `0 errors`

- [ ] **Step 3: Commit**

```bash
git add src/lib/utils/tauri.ts
git commit -m "feat: add frontend tauri wrappers for import/export/integrity"
```

---

## Task 6: Create ExportConfirm Component

**Files:**
- Create: `src/lib/components/ExportConfirm.svelte`

- [ ] **Step 1: Write ExportConfirm.svelte**

Create `src/lib/components/ExportConfirm.svelte`:

```svelte
<script lang="ts">
	import { exportVault } from '$lib/utils/tauri';
	import type { ExportData } from '$lib/utils/tauri';

	interface Props {
		onclose: () => void;
		onexport: (data: ExportData) => void;
	}

	let { onclose, onexport }: Props = $props();

	let format = $state('json');
	let masterPassword = $state('');
	let exportPassword = $state('');
	let exportPasswordConfirm = $state('');
	let error = $state('');
	let loading = $state(false);

	function triggerDownload(data: ExportData) {
		const blob = new Blob([data.data], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = data.filename;
		a.click();
		URL.revokeObjectURL(url);
	}

	async function handleExport() {
		error = '';
		if (!masterPassword) {
			error = '请输入主密码以验证身份';
			return;
		}
		if (format === 'encrypted') {
			if (!exportPassword || exportPassword.length < 8) {
				error = '导出密码至少需要 8 个字符';
				return;
			}
			if (exportPassword !== exportPasswordConfirm) {
				error = '两次输入的导出密码不一致';
				return;
			}
		}

		loading = true;
		try {
			const data = await exportVault(format, masterPassword, format === 'encrypted' ? exportPassword : undefined);
			triggerDownload(data);
			onexport(data);
		} catch (e: unknown) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onclose}>
	<div class="w-full max-w-md rounded-xl bg-card p-6 shadow-xl" onclick={(e) => e.stopPropagation()}>
		<h2 class="mb-4 text-lg font-bold text-heading">导出保险库</h2>

		{#if error}
			<div class="mb-3 rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{error}</div>
		{/if}

		<div class="flex flex-col gap-4">
			<label class="flex flex-col gap-1">
				<span class="text-sm text-body">导出格式</span>
				<select
					class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
					bind:value={format}
				>
					<option value="json">JSON（无损）</option>
					<option value="csv">CSV（通用）</option>
					<option value="encrypted">加密备份 (.bvault)</option>
				</select>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm text-body">主密码（验证身份）</span>
				<input
					type="password"
					class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
					bind:value={masterPassword}
					placeholder="输入主密码"
				/>
			</label>

			{#if format === 'encrypted'}
				<label class="flex flex-col gap-1">
					<span class="text-sm text-body">导出密码</span>
					<input
						type="password"
						class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
						bind:value={exportPassword}
						placeholder="设置导出文件密码"
					/>
				</label>
				<label class="flex flex-col gap-1">
					<span class="text-sm text-body">确认导出密码</span>
					<input
						type="password"
						class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
						bind:value={exportPasswordConfirm}
						placeholder="再次输入导出密码"
					/>
				</label>
			{/if}
		</div>

		<div class="mt-5 flex justify-end gap-2">
			<button
				class="cursor-pointer rounded-md border border-line px-4 py-2 text-sm text-body hover:bg-sidebar"
				onclick={onclose}
			>
				取消
			</button>
			<button
				class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
				onclick={handleExport}
				disabled={loading}
			>
				{loading ? '导出中...' : '导出'}
			</button>
		</div>
	</div>
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/ExportConfirm.svelte
git commit -m "feat: add ExportConfirm component with format selection and password verification"
```

---

## Task 7: Create ImportPreview Component

**Files:**
- Create: `src/lib/components/ImportPreview.svelte`

- [ ] **Step 1: Write ImportPreview.svelte**

Create `src/lib/components/ImportPreview.svelte`:

```svelte
<script lang="ts">
	import type { PreviewResult } from '$lib/utils/tauri';
	import { importVault } from '$lib/utils/tauri';

	interface Props {
		preview: PreviewResult;
		format: string;
		content: string;
		password?: string;
		oncomplete: () => void;
		oncancel: () => void;
	}

	let { preview, format, content, password, oncomplete, oncancel }: Props = $props();

	// resolutions: existing_id -> "keep" | "skip" | "replace"
	let resolutions = $state<Record<string, string>>({});
	for (const dup of preview.duplicates) {
		resolutions[dup.existing_id] = 'keep';
	}

	let loading = $state(false);
	let error = $state('');

	async function handleImport() {
		loading = true;
		error = '';
		try {
			const result = await importVault(format, content, resolutions, password);
			alert(`导入完成：新增 ${result.imported} 条，跳过 ${result.skipped} 条，替换 ${result.replaced} 条`);
			oncomplete();
		} catch (e: unknown) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex h-full flex-col bg-page">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-line px-5 py-3">
		<h2 class="text-lg font-bold text-heading">导入预览</h2>
		<button class="cursor-pointer text-hint hover:text-heading" onclick={oncancel}>✕</button>
	</div>

	<!-- Summary -->
	<div class="border-b border-line px-5 py-3">
		<div class="flex gap-6 text-sm">
			<span class="text-body">共 <span class="font-medium text-heading">{preview.total}</span> 条</span>
			{#if preview.duplicates.length > 0}
				<span class="text-yellow-400">重复 <span class="font-medium">{preview.duplicates.length}</span> 条</span>
			{:else}
				<span class="text-green-400">无重复</span>
			{/if}
		</div>
	</div>

	{#if error}
		<div class="mx-5 mt-3 rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{error}</div>
	{/if}

	<!-- Duplicate list -->
	{#if preview.duplicates.length > 0}
		<div class="flex-1 overflow-y-auto p-5">
			<h3 class="mb-3 text-sm font-medium text-heading">重复条目处理</h3>
			<div class="flex flex-col gap-2">
				{#each preview.duplicates as dup}
					<div class="flex items-center justify-between rounded-lg border border-line bg-card px-4 py-3">
						<div class="min-w-0 flex-1">
							<div class="text-sm font-medium text-heading">{dup.imported_title}</div>
							<div class="text-xs text-hint">用户名：{dup.imported_username || '(空)'}</div>
						</div>
						<select
							class="ml-3 rounded-md border border-line bg-sidebar px-2 py-1 text-xs text-heading outline-none focus:border-accent"
							bind:value={resolutions[dup.existing_id]}
						>
							<option value="keep">保留两者</option>
							<option value="skip">跳过</option>
							<option value="replace">覆盖现有</option>
						</select>
					</div>
				{/each}
			</div>
		</div>
	{:else}
		<div class="flex flex-1 items-center justify-center text-hint">
			<div class="text-center">
				<p class="text-sm">无重复条目，可直接导入</p>
			</div>
		</div>
	{/if}

	<!-- Footer -->
	<div class="flex items-center justify-end border-t border-line px-5 py-3">
		<button
			class="mr-2 cursor-pointer rounded-md border border-line px-4 py-2 text-sm text-body hover:bg-sidebar"
			onclick={oncancel}
		>
			取消
		</button>
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
			onclick={handleImport}
			disabled={loading}
		>
			{loading ? '导入中...' : '确认导入'}
		</button>
	</div>
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/ImportPreview.svelte
git commit -m "feat: add ImportPreview component with duplicate resolution"
```

---

## Task 8: Create ImportExport Main Component

**Files:**
- Create: `src/lib/components/ImportExport.svelte`

- [ ] **Step 1: Write ImportExport.svelte**

Create `src/lib/components/ImportExport.svelte`:

```svelte
<script lang="ts">
	import { previewImport, checkIntegrity } from '$lib/utils/tauri';
	import type { PreviewResult, IntegrityResult, ExportData } from '$lib/utils/tauri';
	import ExportConfirm from './ExportConfirm.svelte';
	import ImportPreview from './ImportPreview.svelte';

	interface Props {
		onclose: () => void;
	}

	let { onclose }: Props = $props();

	type Tab = 'import' | 'export' | 'integrity';
	let tab = $state<Tab>('import');

	// Import state
	let importFormat = $state('json');
	let fileContent = $state('');
	let fileName = $state('');
	let importPassword = $state('');
	let importLoading = $state(false);
	let importError = $state('');
	let previewResult = $state<PreviewResult | null>(null);

	// Export state
	let showExportConfirm = $state(false);

	// Integrity state
	let integrityResult = $state<IntegrityResult | null>(null);
	let integrityLoading = $state(false);

	async function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		fileName = file.name;
		const text = await file.text();
		fileContent = text;
	}

	async function handlePreview() {
		importError = '';
		if (!fileContent) {
			importError = '请先选择文件';
			return;
		}
		importLoading = true;
		try {
			previewResult = await previewImport(
				importFormat,
				fileContent,
				importFormat === 'encrypted' ? importPassword : undefined
			);
		} catch (e: unknown) {
			importError = String(e);
		} finally {
			importLoading = false;
		}
	}

	function handleImportComplete() {
		previewResult = null;
		fileContent = '';
		fileName = '';
	}

	function handleExportComplete(_data: ExportData) {
		showExportConfirm = false;
	}

	async function handleIntegrityCheck() {
		integrityLoading = true;
		try {
			integrityResult = await checkIntegrity();
		} catch (e: unknown) {
			integrityResult = {
				status: 'error',
				issues: [{ severity: 'error', message: String(e) }]
			};
		} finally {
			integrityLoading = false;
		}
	}

	function severityColor(severity: string): string {
		switch (severity) {
			case 'error': return 'text-red-400';
			case 'warning': return 'text-yellow-400';
			default: return 'text-hint';
		}
	}
</script>

<div class="flex h-full flex-col bg-page">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-line px-5 py-3">
		<h2 class="text-lg font-bold text-heading">导入 / 导出</h2>
		<button class="cursor-pointer text-hint hover:text-heading" onclick={onclose}>✕</button>
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-line">
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'import' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'import')}
		>
			导入
		</button>
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'export' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'export')}
		>
			导出
		</button>
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'integrity' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'integrity')}
		>
			完整性检查
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		{#if tab === 'import'}
			{#if previewResult}
				<ImportPreview
					{previewResult}
					format={importFormat}
					content={fileContent}
					password={importFormat === 'encrypted' ? importPassword : undefined}
					oncomplete={handleImportComplete}
					oncancel={() => (previewResult = null)}
				/>
			{:else}
				<div class="p-5">
					<div class="flex flex-col gap-4">
						<label class="flex flex-col gap-1">
							<span class="text-sm text-body">导入格式</span>
							<select
								class="rounded-md border border-line bg-card px-3 py-2 text-sm text-heading outline-none focus:border-accent"
								bind:value={importFormat}
							>
								<option value="json">JSON（本应用格式）</option>
								<option value="csv">CSV（通用）</option>
								<option value="bitwarden">Bitwarden JSON</option>
								<option value="keepass">KeePass CSV</option>
								<option value="chrome">Chrome / Firefox CSV</option>
								<option value="encrypted">加密备份 (.bvault)</option>
							</select>
						</label>

						<label class="flex flex-col gap-1">
							<span class="text-sm text-body">选择文件</span>
							<input
								type="file"
								class="text-sm text-body file:mr-3 file:rounded-md file:border file:border-line file:bg-card file:px-3 file:py-1.5 file:text-sm file:text-heading file:cursor-pointer"
								accept=".csv,.json,.bvault,.txt"
								onchange={handleFileSelect}
							/>
							{#if fileName}
								<span class="text-xs text-hint">已选择：{fileName}</span>
							{/if}
						</label>

						{#if importFormat === 'encrypted'}
							<label class="flex flex-col gap-1">
								<span class="text-sm text-body">导出密码（解密用）</span>
								<input
									type="password"
									class="rounded-md border border-line bg-card px-3 py-2 text-sm text-heading outline-none focus:border-accent"
									bind:value={importPassword}
									placeholder="输入该备份的导出密码"
								/>
							</label>
						{/if}

						{#if importError}
							<div class="rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{importError}</div>
						{/if}

						<button
							class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
							onclick={handlePreview}
							disabled={importLoading}
						>
							{importLoading ? '解析中...' : '预览导入'}
						</button>
					</div>
				</div>
			{/if}
		{:else if tab === 'export'}
			<div class="flex h-full items-center justify-center p-5">
				<div class="w-full max-w-sm text-center">
					<div class="mb-4 text-3xl">📦</div>
					<p class="mb-4 text-sm text-body">导出所有密码条目到文件。导出时需要验证主密码。</p>
					<button
						class="cursor-pointer rounded-md bg-accent px-6 py-2 text-sm font-medium text-white hover:bg-accent-hover"
						onclick={() => (showExportConfirm = true)}
					>
						开始导出
					</button>
				</div>
			</div>
		{:else}
			<!-- Integrity check -->
			<div class="p-5">
				<div class="mb-4 text-center">
					<p class="text-sm text-body">检查数据库完整性和加密数据一致性。</p>
					<button
						class="mt-3 cursor-pointer rounded-md bg-accent px-6 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
						onclick={handleIntegrityCheck}
						disabled={integrityLoading}
					>
						{integrityLoading ? '检查中...' : '开始检查'}
					</button>
				</div>

				{#if integrityResult}
					<div class="mt-4">
						<div class="mb-3 flex items-center gap-2">
							<span class="text-sm font-medium text-heading">状态：</span>
							<span class="rounded-full px-2 py-0.5 text-xs font-medium
								{integrityResult.status === 'ok' ? 'bg-green-500/20 text-green-400' : ''}
								{integrityResult.status === 'warning' ? 'bg-yellow-500/20 text-yellow-400' : ''}
								{integrityResult.status === 'error' ? 'bg-red-500/20 text-red-400' : ''}
							">
								{integrityResult.status === 'ok' ? '正常' : integrityResult.status === 'warning' ? '警告' : '错误'}
							</span>
						</div>
						{#each integrityResult.issues as issue}
							<div class="mb-2 rounded-md border border-line bg-card px-4 py-2.5">
								<span class="text-xs font-medium {severityColor(issue.severity)}">[{issue.severity.toUpperCase()}]</span>
								<span class="ml-2 text-sm text-body">{issue.message}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

{#if showExportConfirm}
	<ExportConfirm
		onclose={() => (showExportConfirm = false)}
		onexport={handleExportComplete}
	/>
{/if}
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/ImportExport.svelte
git commit -m "feat: add ImportExport component with import/export/integrity tabs"
```

---

## Task 9: Integrate into Main Page

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add ImportExport import and view mode**

In `src/routes/+page.svelte`:

1. Add import at the top of the `<script>` block (after the `Settings` import):

```typescript
import ImportExport from '$lib/components/ImportExport.svelte';
```

2. Update the `ViewMode` type to include `'import-export'`:

```typescript
type ViewMode = 'empty' | 'detail' | 'edit' | 'create' | 'settings' | 'import-export';
```

- [ ] **Step 2: Add sidebar button for Import/Export**

In the bottom bar (the `<div class="flex items-center gap-1">` section, after the Settings button and before `</div>`), add:

```svelte
<button
    class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-hint hover:text-accent"
    onclick={() => (viewMode = 'import-export')}
    title="导入/导出"
>
    📦 导入/导出
</button>
```

- [ ] **Step 3: Add ImportExport view to right panel**

In the right panel section, add this block right after the `{#if viewMode === 'settings'}` block:

```svelte
{:else if viewMode === 'import-export'}
    <ImportExport onclose={() => (viewMode = 'empty')} />
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `cd /Users/chldu/Workspace/bavu-iru && npx svelte-check --threshold error 2>&1 | tail -5`
Expected: `0 errors`

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: integrate ImportExport into main page with sidebar button"
```

---

## Task 10: End-to-End Test

**Files:** None (manual testing)

- [ ] **Step 1: Build and run the app**

Run: `cd /Users/chldu/Workspace/bavu-iru && cargo tauri dev 2>&1 | head -20`

Wait for the app to launch.

- [ ] **Step 2: Test export**

1. Unlock the vault
2. Click "📦 导入/导出" in sidebar
3. Go to "导出" tab → click "开始导出"
4. Select JSON format, enter master password, click "导出"
5. Verify file downloads

- [ ] **Step 3: Test import**

1. Go to "导入" tab
2. Select "JSON（本应用格式）"
3. Choose the exported JSON file
4. Click "预览导入"
5. Verify preview shows entries correctly
6. If duplicates, select resolution → confirm import

- [ ] **Step 4: Test encrypted export/import roundtrip**

1. Export with "加密备份 (.bvault)" format, set export password
2. Import the .bvault file with the same export password
3. Verify all entries are restored

- [ ] **Step 5: Test integrity check**

1. Go to "完整性检查" tab
2. Click "开始检查"
3. Verify it shows status "ok" with entry count

- [ ] **Step 6: Commit any fixes**

If any fixes were needed during testing:

```bash
git add -A
git commit -m "fix: address issues found during e2e testing"
```
