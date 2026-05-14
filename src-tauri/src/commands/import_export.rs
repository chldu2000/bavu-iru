use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub filename: String,
    pub data: String,
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
    let magic = &data[0..4];
    if magic != b"BVLT" {
        return Err(AppError::Import("Not a valid .bvault file".into()));
    }
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
    let decrypted = cipher::decrypt(&key, nonce_and_ciphertext)
        .map_err(|_| AppError::Import("Wrong password or corrupted file".into()))?;
    let json_str = String::from_utf8(decrypted)
        .map_err(|e| AppError::Import(format!("Invalid UTF-8: {}", e)))?;
    parse_native_json(&json_str)
}

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

            let mut output = Vec::with_capacity(8 + encrypted.len());
            output.extend_from_slice(b"BVLT");
            output.extend_from_slice(&1u16.to_be_bytes());
            output.extend_from_slice(&32u16.to_be_bytes());
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
                _ => {}
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
        strength: imp.password.as_ref().map(|p| crate::crypto::strength::evaluate_strength(p).score as i32),
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

    // Check 2: entries have non-empty titles
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

        // Check 3: orphan entries (folder_id references)
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

        // Check 4: orphan tag relations
        if let Ok(tags) = db.list_tags() {
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
        }
    }

    let has_errors = issues.iter().any(|i| i.severity == "error");
    let status = if has_errors { "error" } else if issues.iter().any(|i| i.severity == "warning") { "warning" } else { "ok" };

    Ok(IntegrityResult { status: status.into(), issues })
}
