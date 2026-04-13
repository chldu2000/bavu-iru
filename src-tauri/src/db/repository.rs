use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::Mutex;

use super::models::{Entry, Folder};

pub struct Database {
	conn: Mutex<Connection>,
}

impl Database {
	/// Open (or create) the SQLite database at the given path.
	pub fn open(path: &Path) -> SqliteResult<Self> {
		let conn = Connection::open(path)?;
		conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
		let db = Self {
			conn: Mutex::new(conn),
		};
		db.run_migrations()?;
		Ok(db)
	}

	/// Create an in-memory database (useful for testing).
	pub fn open_in_memory() -> SqliteResult<Self> {
		let conn = Connection::open_in_memory()?;
		conn.execute_batch("PRAGMA foreign_keys=ON;")?;
		let db = Self {
			conn: Mutex::new(conn),
		};
		db.run_migrations()?;
		Ok(db)
	}

	fn run_migrations(&self) -> SqliteResult<()> {
		let conn = self.conn.lock().unwrap();
		conn.execute_batch(
			"
            CREATE TABLE IF NOT EXISTS meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS folders (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                parent_id  TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id)
            );

            CREATE TABLE IF NOT EXISTS entries (
                id            TEXT PRIMARY KEY,
                folder_id     TEXT,
                title         TEXT NOT NULL,
                username      TEXT,
                password      TEXT,
                url           TEXT,
                notes         TEXT,
                custom_fields TEXT,
                tags          TEXT,
                strength      INTEGER,
                expires_at    TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES folders(id)
            );
            "
        )?;
		Ok(())
	}

	// --- Meta operations ---

	pub fn set_meta(&self, key: &str, value: &str) -> SqliteResult<()> {
		let conn = self.conn.lock().unwrap();
		conn.execute(
			"INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
			rusqlite::params![key, value],
		)?;
		Ok(())
	}

	pub fn get_meta(&self, key: &str) -> SqliteResult<Option<String>> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
		let mut rows = stmt.query(rusqlite::params![key])?;
		match rows.next()? {
			Some(row) => Ok(Some(row.get(0)?)),
			None => Ok(None),
		}
	}

	// --- Entry operations ---

	pub fn create_entry(&self, entry: &Entry) -> SqliteResult<()> {
		let conn = self.conn.lock().unwrap();
		conn.execute(
			"INSERT INTO entries (id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
			rusqlite::params![
				entry.id, entry.folder_id, entry.title, entry.username,
				entry.password, entry.url, entry.notes, entry.custom_fields,
				entry.tags, entry.strength, entry.expires_at,
				entry.created_at, entry.updated_at
			],
		)?;
		Ok(())
	}

	pub fn list_entries(&self) -> SqliteResult<Vec<Entry>> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn.prepare(
			"SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, created_at, updated_at
             FROM entries ORDER BY updated_at DESC"
		)?;
		let entries = stmt.query_map([], |row| {
			Ok(Entry {
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
				created_at: row.get(11)?,
				updated_at: row.get(12)?,
			})
		})?;
		entries.collect()
	}

	pub fn get_entry(&self, id: &str) -> SqliteResult<Option<Entry>> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn.prepare(
			"SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, created_at, updated_at
             FROM entries WHERE id = ?1"
		)?;
		let mut rows = stmt.query(rusqlite::params![id])?;
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
				created_at: row.get(11)?,
				updated_at: row.get(12)?,
			})),
			None => Ok(None),
		}
	}

	pub fn delete_entry(&self, id: &str) -> SqliteResult<bool> {
		let conn = self.conn.lock().unwrap();
		let affected = conn.execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])?;
		Ok(affected > 0)
	}

	pub fn update_entry(&self, entry: &Entry) -> SqliteResult<bool> {
		let conn = self.conn.lock().unwrap();
		let affected = conn.execute(
			"UPDATE entries SET folder_id=?2, title=?3, username=?4, password=?5, url=?6, notes=?7, custom_fields=?8, tags=?9, strength=?10, expires_at=?11, updated_at=?12
             WHERE id=?1",
			rusqlite::params![
				entry.id, entry.folder_id, entry.title, entry.username,
				entry.password, entry.url, entry.notes, entry.custom_fields,
				entry.tags, entry.strength, entry.expires_at, entry.updated_at
			],
		)?;
		Ok(affected > 0)
	}

	// --- Folder operations ---

	pub fn create_folder(&self, folder: &Folder) -> SqliteResult<()> {
		let conn = self.conn.lock().unwrap();
		conn.execute(
			"INSERT INTO folders (id, name, parent_id, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			rusqlite::params![
				folder.id, folder.name, folder.parent_id,
				folder.sort_order, folder.created_at, folder.updated_at
			],
		)?;
		Ok(())
	}

	pub fn list_folders(&self) -> SqliteResult<Vec<Folder>> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn.prepare(
			"SELECT id, name, parent_id, sort_order, created_at, updated_at
             FROM folders ORDER BY sort_order, name"
		)?;
		let folders = stmt.query_map([], |row| {
			Ok(Folder {
				id: row.get(0)?,
				name: row.get(1)?,
				parent_id: row.get(2)?,
				sort_order: row.get(3)?,
				created_at: row.get(4)?,
				updated_at: row.get(5)?,
			})
		})?;
		folders.collect()
	}

	pub fn delete_folder(&self, id: &str) -> SqliteResult<bool> {
		let conn = self.conn.lock().unwrap();
		let affected = conn.execute("DELETE FROM folders WHERE id = ?1", rusqlite::params![id])?;
		Ok(affected > 0)
	}
}
