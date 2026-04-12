use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

pub struct Repository {
    conn: Connection,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let repo = Self { conn };
        repo.init_schema()?;
        Ok(repo)
    }

    fn init_schema(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                folder_id TEXT,
                encrypted_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES folders(id)
            );

            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> SqliteResult<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?")?;
        let result = stmt.query_row([key], |row| row.get(0));
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_meta(&self, key: &str, value: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?, ?)",
            [key, value],
        )?;
        Ok(())
    }

    pub fn insert_entry(&self, id: &str, folder_id: Option<&str>, encrypted_data: &str, created_at: &str, updated_at: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO entries (id, folder_id, encrypted_data, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![id, folder_id, encrypted_data, created_at, updated_at],
        )?;
        Ok(())
    }

    pub fn update_entry(&self, id: &str, folder_id: Option<&str>, encrypted_data: &str, updated_at: &str) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE entries SET folder_id = ?, encrypted_data = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![folder_id, encrypted_data, updated_at, id],
        )?;
        Ok(())
    }

    pub fn delete_entry(&self, id: &str) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM entries WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn get_entry(&self, id: &str) -> SqliteResult<Option<(String, Option<String>, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, folder_id, encrypted_data, created_at, updated_at FROM entries WHERE id = ?"
        )?;
        let result = stmt.query_row([id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        });
        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn list_entries(&self) -> SqliteResult<Vec<(String, Option<String>, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, folder_id, encrypted_data, created_at, updated_at FROM entries ORDER BY updated_at DESC"
        )?;
        let entries = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?;
        entries.collect()
    }

    pub fn search_entries(&self, _query: &str) -> SqliteResult<Vec<(String, Option<String>, String, String, String)>> {
        // Search is done post-decryption in the application layer
        self.list_entries()
    }

    pub fn insert_folder(&self, id: &str, name: &str, parent_id: Option<&str>, sort_order: i32, created_at: &str, updated_at: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO folders (id, name, parent_id, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![id, name, parent_id, sort_order, created_at, updated_at],
        )?;
        Ok(())
    }

    pub fn update_folder(&self, id: &str, name: &str, parent_id: Option<&str>, sort_order: i32, updated_at: &str) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE folders SET name = ?, parent_id = ?, sort_order = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![name, parent_id, sort_order, updated_at, id],
        )?;
        Ok(())
    }

    pub fn delete_folder(&self, id: &str) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM folders WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn list_folders(&self) -> SqliteResult<Vec<(String, String, Option<String>, i32, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM folders ORDER BY sort_order"
        )?;
        let folders = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        folders.collect()
    }
}