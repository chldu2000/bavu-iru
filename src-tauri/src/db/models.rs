use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
	pub id: String,
	pub folder_id: Option<String>,
	pub title: String,
	pub username: Option<String>,
	pub password: Option<String>,
	pub url: Option<String>,
	pub notes: Option<String>,
	pub custom_fields: Option<String>,
	pub tags: Option<String>,
	pub strength: Option<i32>,
	pub expires_at: Option<String>,
	pub is_favorite: bool,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
	pub id: String,
	pub name: String,
	pub color: String,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
	pub id: String,
	pub name: String,
	pub parent_id: Option<String>,
	pub sort_order: i32,
	pub created_at: String,
	pub updated_at: String,
}

/// Metadata key-value pairs for vault configuration (KDF params, salt, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
	pub key: String,
	pub value: String,
}
