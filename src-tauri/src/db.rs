use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use log::{error, info};
use rusqlite::{params, types::Type, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tauri::Manager;

use crate::pin_cache::PinCache;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    current_pin_key: Arc<Mutex<Option<Vec<u8>>>>,
    pin_cache: Arc<PinCache>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiInfo {
    pub id: i64,
    pub profile_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub api_url: String,
    pub port: u16,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DashboardWidgetPref {
    pub widget_key: String,
    pub visible: bool,
    pub position: i32,
}

impl Database {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_dir = app_handle
            .path()
            .app_local_data_dir()
            .expect("Failed to get app local data dir");
        std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

        let db_path = app_dir.join("app.db");

        let conn = Connection::open(db_path)?;

        let pin_cache = Arc::new(PinCache::new());

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
            current_pin_key: Arc::new(Mutex::new(None)),
            pin_cache,
        };
        db.initialize_tables()?;
        db.migrate_data()?;

        Ok(db)
    }

    fn initialize_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS first_run (
                id INTEGER PRIMARY KEY,
                has_run BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                id INTEGER PRIMARY KEY,
                password_hash TEXT NOT NULL,
                pin_salt TEXT NOT NULL DEFAULT ''
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS dashboard_preferences (
                id INTEGER PRIMARY KEY,
                profile_id INTEGER NOT NULL,
                widget_key TEXT NOT NULL,
                visible BOOLEAN NOT NULL DEFAULT 1,
                position INTEGER NOT NULL,
                FOREIGN KEY(profile_id) REFERENCES api_info(id)
            )",
            [],
        )?;

        Ok(())
    }

    fn derive_encryption_key(&self, pin: &str, salt: &str) -> Result<Vec<u8>, String> {
        let salt = SaltString::from_b64(salt).map_err(|e| format!("Invalid salt: {}", e))?;

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(pin.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash PIN for encryption key: {}", e))?;

        let hash_bytes = password_hash
            .hash
            .ok_or_else(|| "Failed to get hash bytes".to_string())?
            .as_bytes()
            .to_vec();

        Ok(hash_bytes)
    }

    fn set_current_pin_key(&self, key: Vec<u8>) {
        let mut current_key = self.current_pin_key.lock().unwrap();
        *current_key = Some(key);
    }

    fn encrypt_string(&self, plaintext: &str, pin: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        use rand::{thread_rng, Rng};

        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        let key_bytes = hasher.finalize();
        let key = Key::from_slice(&key_bytes);

        let cipher = ChaCha20Poly1305::new(key);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn decrypt_string(
        &self,
        ciphertext: &[u8],
        nonce_bytes: &[u8],
        pin: &str,
    ) -> Result<String, String> {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        let key_bytes = hasher.finalize();
        let key = Key::from_slice(&key_bytes);

        let cipher = ChaCha20Poly1305::new(key);

        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
    }

    fn migrate_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let has_api_key_column: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='api_key'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        let has_encrypted_api_key_column: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if has_api_key_column && !has_encrypted_api_key_column {
            info!("Migrating from unencrypted to encrypted API info schema");

            conn.execute(
                "CREATE TABLE api_info_new (
                    id INTEGER PRIMARY KEY,
                    profile_name TEXT NOT NULL UNIQUE,
                    encrypted_api_key BLOB NOT NULL,
                    api_key_nonce BLOB NOT NULL,
                    encrypted_api_secret BLOB NOT NULL,
                    api_secret_nonce BLOB NOT NULL,
                    api_url TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT 0
                )",
                [],
            )?;
        }

        let has_pin_salt_column: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('app_settings') WHERE name='pin_salt'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if !has_pin_salt_column {
            conn.execute(
                "ALTER TABLE app_settings ADD COLUMN pin_salt TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        Ok(())
    }

    fn complete_migration(&self, pin: &str) -> Result<(), String> {
        info!("Starting complete_migration process");

        let (has_api_key_column, has_encrypted_api_key_column, has_api_info_new) =
            {
                let conn = self.conn.lock().unwrap();

                let has_api_key = conn
                    .query_row(
                        "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='api_key'",
                        [],
                        |row| {
                            let count: i64 = row.get(0)?;
                            Ok(count > 0)
                        },
                    )
                    .map_err(|e| format!("Failed to check for api_key column: {}", e))?;

                let has_encrypted_api_key = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            ).map_err(|e| format!("Failed to check for encrypted_api_key column: {}", e))?;

                let has_new_table = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_info_new'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            ).map_err(|e| format!("Failed to check for api_info_new table: {}", e))?;

                (has_api_key, has_encrypted_api_key, has_new_table)
            };

        info!(
            "Migration check: has_api_key={}, has_encrypted_api_key={}, has_api_info_new={}",
            has_api_key_column, has_encrypted_api_key_column, has_api_info_new
        );

        if has_encrypted_api_key_column || !has_api_key_column {
            info!("No migration needed - schema already using encryption or no data to migrate");
            return Ok(());
        }

        {
            let conn = self.conn.lock().unwrap();

            info!("Setting up salt for encryption");
            let salt_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM app_settings WHERE id = 1 AND pin_salt != ''",
                    [],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )
                .map_err(|e| format!("Failed to check for salt: {}", e))?;

            if !salt_exists {
                info!("Creating new salt for encryption");
                let salt = SaltString::generate(&mut OsRng).to_string();
                conn.execute(
                    "UPDATE app_settings SET pin_salt = ?1 WHERE id = 1",
                    params![salt],
                )
                .map_err(|e| format!("Failed to update salt: {}", e))?;
            }
        }

        if !has_api_info_new {
            let conn = self.conn.lock().unwrap();

            info!("Creating new table for encrypted data");
            conn.execute(
                "CREATE TABLE api_info_new (
                    id INTEGER PRIMARY KEY,
                    profile_name TEXT NOT NULL UNIQUE,
                    encrypted_api_key BLOB NOT NULL,
                    api_key_nonce BLOB NOT NULL,
                    encrypted_api_secret BLOB NOT NULL,
                    api_secret_nonce BLOB NOT NULL,
                    api_url TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT 0
                )",
                [],
            )
            .map_err(|e| format!("Failed to create new table: {}", e))?;
        }

        let profiles = {
            let conn = self.conn.lock().unwrap();

            info!("Reading data from unencrypted table");
            let mut stmt = conn.prepare(
                "SELECT id, profile_name, api_key, api_secret, api_url, port, is_default FROM api_info"
            ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, bool>(6)?,
                    ))
                })
                .map_err(|e| format!("Failed to query old data: {}", e))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to collect profiles: {}", e))?
        };

        info!("Found {} profiles to migrate", profiles.len());

        for (id, profile_name, api_key, api_secret, api_url, port, is_default) in profiles {
            info!("Encrypting data for profile: {}", profile_name);
            let (encrypted_api_key, api_key_nonce) = self.encrypt_string(&api_key, pin)?;
            let (encrypted_api_secret, api_secret_nonce) = self.encrypt_string(&api_secret, pin)?;

            {
                let conn = self.conn.lock().unwrap();
                info!("Inserting encrypted data for profile: {}", profile_name);
                conn.execute(
                    "INSERT INTO api_info_new (id, profile_name, encrypted_api_key, api_key_nonce, 
                     encrypted_api_secret, api_secret_nonce, api_url, port, is_default) 
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        id,
                        profile_name,
                        encrypted_api_key,
                        api_key_nonce,
                        encrypted_api_secret,
                        api_secret_nonce,
                        api_url,
                        port,
                        is_default
                    ],
                )
                .map_err(|e| format!("Failed to insert encrypted data: {}", e))?;
            }
        }

        {
            let conn = self.conn.lock().unwrap();

            info!("Finalizing migration by replacing tables");
            conn.execute("DROP TABLE api_info", [])
                .map_err(|e| format!("Failed to drop old table: {}", e))?;

            conn.execute("ALTER TABLE api_info_new RENAME TO api_info", [])
                .map_err(|e| format!("Failed to rename new table: {}", e))?;
        }

        info!("Migration to encrypted API info completed successfully");
        Ok(())
    }

    pub fn is_first_run(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM first_run", [], |row| row.get(0))?;
        Ok(count == 0)
    }

    pub fn set_has_run(&self) -> Result<()> {
        info!("Entering set_has_run");
        let conn = self.conn.lock().unwrap();
        match conn.execute(
            "INSERT OR REPLACE INTO first_run (id, has_run) VALUES (1, 1)",
            [],
        ) {
            Ok(rows) => {
                info!("Inserted/Updated {} row(s) in first_run table", rows);
                Ok(())
            }
            Err(e) => {
                error!("Failed to insert/update first_run table: {}", e);
                Err(e)
            }
        }
    }

    pub fn save_initial_api_info(&self, api_info: &ApiInfo) -> Result<()> {
        info!("Entering save_initial_api_info for first-time setup");
        let conn = self.conn.lock().unwrap();

        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_info'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if table_exists {
            let has_encrypted_columns: bool = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )?;

            if has_encrypted_columns {
                info!("Dropping existing encrypted table for fresh setup");
                conn.execute("DROP TABLE api_info", [])?;
            }
        }

        info!("Creating fresh unencrypted table for initial setup");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_info (
                id INTEGER PRIMARY KEY,
                profile_name TEXT NOT NULL UNIQUE,
                api_key TEXT NOT NULL,
                api_secret TEXT NOT NULL,
                api_url TEXT NOT NULL,
                port INTEGER NOT NULL,
                is_default BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        info!("Inserting first profile with unencrypted schema");
        conn.execute(
            "INSERT INTO api_info (profile_name, api_key, api_secret, api_url, port, is_default) 
            VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![
                api_info.profile_name,
                api_info.api_key,
                api_info.api_secret,
                api_info.api_url,
                api_info.port
            ],
        )?;

        info!("Initial API info saved successfully");
        Ok(())
    }
    pub fn save_api_info(&self, api_info: &ApiInfo) -> Result<()> {
        info!(
            "Entering save_api_info for profile: {}",
            api_info.profile_name
        );
        let conn = self.conn.lock().unwrap();

        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_info'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        let has_encrypted_columns: bool = if table_exists {
            conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )?
        } else {
            false
        };

        let has_unencrypted_columns: bool = if table_exists {
            conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='api_key'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )?
        } else {
            false
        };

        let is_first_profile: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='api_info'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count == 0)
            },
        )?;

        info!("Table exists: {}, Has encrypted columns: {}, Has unencrypted columns: {}, Is first profile: {}", 
              table_exists, has_encrypted_columns, has_unencrypted_columns, is_first_profile);

        if !table_exists {
            info!("Creating new unencrypted api_info table");
            conn.execute(
                "CREATE TABLE api_info (
                    id INTEGER PRIMARY KEY,
                    profile_name TEXT NOT NULL UNIQUE,
                    api_key TEXT NOT NULL,
                    api_secret TEXT NOT NULL,
                    api_url TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT 0
                )",
                [],
            )?;

            info!("Inserting first profile with unencrypted schema");
            conn.execute(
                "INSERT INTO api_info (profile_name, api_key, api_secret, api_url, port, is_default) 
                 VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                params![api_info.profile_name,api_info.api_key,api_info.api_secret,api_info.api_url,api_info.port],
            )?;

            info!("First profile inserted successfully");
            return Ok(());
        }

        if has_unencrypted_columns {
            info!("Using existing unencrypted schema");
            conn.execute(
                "INSERT OR REPLACE INTO api_info (profile_name, api_key, api_secret, api_url, port, is_default) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![api_info.profile_name,api_info.api_key,api_info.api_secret,api_info.api_url,api_info.port,api_info.is_default],
            )?;
        } else if has_encrypted_columns {
            info!("Using existing encrypted schema");

            let pin_hash: Option<String> = conn
                .query_row(
                    "SELECT password_hash FROM app_settings WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .optional()?;

            if pin_hash.is_none() {
                return Err(rusqlite::Error::InvalidParameterName(
                    "No PIN available for encryption".to_string(),
                )
                .into());
            }

            let pin = self.get_cached_pin().map_err(|e| {
                error!("Failed to get cached PIN: {}", e);
                rusqlite::Error::InvalidParameterName(
                    "Failed to get PIN for encryption".to_string(),
                )
            })?;

            let (encrypted_api_key, api_key_nonce) =
                self.encrypt_string(&api_info.api_key, &pin).map_err(|e| {
                    error!("Failed to encrypt API key: {}", e);
                    rusqlite::Error::InvalidParameterName("Failed to encrypt API key".to_string())
                })?;

            let (encrypted_api_secret, api_secret_nonce) = self
                .encrypt_string(&api_info.api_secret, &pin)
                .map_err(|e| {
                    error!("Failed to encrypt API secret: {}", e);
                    rusqlite::Error::InvalidParameterName(
                        "Failed to encrypt API secret".to_string(),
                    )
                })?;

            conn.execute(
                "INSERT OR REPLACE INTO api_info (profile_name, encrypted_api_key, api_key_nonce, 
                 encrypted_api_secret, api_secret_nonce, api_url, port, is_default) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    api_info.profile_name,
                    encrypted_api_key,
                    api_key_nonce,
                    encrypted_api_secret,
                    api_secret_nonce,
                    api_info.api_url,
                    api_info.port,
                    api_info.is_default
                ],
            )?;
        } else {
            info!("Recreating table with unencrypted schema");
            conn.execute("DROP TABLE IF EXISTS api_info", [])?;
            conn.execute(
                "CREATE TABLE api_info (
                    id INTEGER PRIMARY KEY,
                    profile_name TEXT NOT NULL UNIQUE,
                    api_key TEXT NOT NULL,
                    api_secret TEXT NOT NULL,
                    api_url TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT 0
                )",
                [],
            )?;

            conn.execute(
                "INSERT INTO api_info (profile_name, api_key, api_secret, api_url, port, is_default) 
                 VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                params![api_info.profile_name,api_info.api_key,api_info.api_secret,api_info.api_url,api_info.port],
            )?;
        }

        info!("save_api_info completed successfully");
        Ok(())
    }

    pub fn set_default_profile(&self, profile_name: &str) -> Result<()> {
        info!("Entering set_default_profile for profile: {}", profile_name);
        let mut conn = self.conn.lock().unwrap();

        let tx = conn.transaction()?;

        info!("Resetting all profiles to non-default");
        tx.execute("UPDATE api_info SET is_default = 0", [])?;

        info!("Setting new default profile");
        let rows_affected = tx.execute(
            "UPDATE api_info SET is_default = 1 WHERE profile_name = ?1",
            params![profile_name],
        )?;

        if rows_affected == 0 {
            tx.rollback()?;
            return Err(rusqlite::Error::QueryReturnedNoRows.into());
        }

        tx.commit()?;

        info!("set_default_profile completed successfully");
        Ok(())
    }

    fn get_cached_pin(&self) -> Result<String, String> {
        if let Some(pin) = self.pin_cache.get_pin() {
            log::info!(
                "Successfully retrieved PIN from cache, length: {}",
                pin.len()
            );
            return Ok(pin);
        }

        log::error!("PIN not found in cache");
        Err("User needs to authenticate first".to_string())
    }

    pub fn get_api_info(&self, profile_name: Option<&str>) -> Result<Option<ApiInfo>> {
        let conn = self.conn.lock().unwrap();

        let has_encrypted_columns: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if has_encrypted_columns {
            let query = match profile_name {
                Some(_) => "SELECT id, profile_name, encrypted_api_key, api_key_nonce, encrypted_api_secret, api_secret_nonce, api_url, port, is_default FROM api_info WHERE profile_name = ?1",
                None => "SELECT id, profile_name, encrypted_api_key, api_key_nonce, encrypted_api_secret, api_secret_nonce, api_url, port, is_default FROM api_info WHERE is_default = 1",
            };

            let mut stmt = conn.prepare(query)?;

            let result = if let Some(name) = profile_name {
                stmt.query_row(params![name], |row| {
                    let id: i64 = row.get(0)?;
                    let profile_name: String = row.get(1)?;
                    let encrypted_api_key: Vec<u8> = row.get(2)?;
                    let api_key_nonce: Vec<u8> = row.get(3)?;
                    let encrypted_api_secret: Vec<u8> = row.get(4)?;
                    let api_secret_nonce: Vec<u8> = row.get(5)?;
                    let api_url: String = row.get(6)?;
                    let port: u16 = row.get(7)?;
                    let is_default: bool = row.get(8)?;

                    Ok((
                        id,
                        profile_name,
                        encrypted_api_key,
                        api_key_nonce,
                        encrypted_api_secret,
                        api_secret_nonce,
                        api_url,
                        port,
                        is_default,
                    ))
                })
            } else {
                stmt.query_row([], |row| {
                    let id: i64 = row.get(0)?;
                    let profile_name: String = row.get(1)?;
                    let encrypted_api_key: Vec<u8> = row.get(2)?;
                    let api_key_nonce: Vec<u8> = row.get(3)?;
                    let encrypted_api_secret: Vec<u8> = row.get(4)?;
                    let api_secret_nonce: Vec<u8> = row.get(5)?;
                    let api_url: String = row.get(6)?;
                    let port: u16 = row.get(7)?;
                    let is_default: bool = row.get(8)?;

                    Ok((
                        id,
                        profile_name,
                        encrypted_api_key,
                        api_key_nonce,
                        encrypted_api_secret,
                        api_secret_nonce,
                        api_url,
                        port,
                        is_default,
                    ))
                })
            };

            match result {
                Ok((
                    id,
                    profile_name,
                    encrypted_api_key,
                    api_key_nonce,
                    encrypted_api_secret,
                    api_secret_nonce,
                    api_url,
                    port,
                    is_default,
                )) => {
                    let pin = match self.get_cached_pin() {
                        Ok(pin) => pin,
                        Err(_) => {
                            return Ok(Some(ApiInfo {
                                id,
                                profile_name,
                                api_key: String::new(),
                                api_secret: String::new(),
                                api_url,
                                port,
                                is_default,
                            }));
                        }
                    };

                    // Decrypt the API key and secret
                    let api_key =
                        match self.decrypt_string(&encrypted_api_key, &api_key_nonce, &pin) {
                            Ok(decrypted) => decrypted,
                            Err(e) => {
                                error!("Failed to decrypt API key: {}", e);
                                String::new()
                            }
                        };

                    let api_secret =
                        match self.decrypt_string(&encrypted_api_secret, &api_secret_nonce, &pin) {
                            Ok(decrypted) => decrypted,
                            Err(e) => {
                                error!("Failed to decrypt API secret: {}", e);
                                String::new()
                            }
                        };

                    Ok(Some(ApiInfo {
                        id,
                        profile_name,
                        api_key,
                        api_secret,
                        api_url,
                        port,
                        is_default,
                    }))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        } else {
            let query = match profile_name {
                Some(_) => "SELECT id, profile_name, api_key, api_secret, api_url, port, is_default FROM api_info WHERE profile_name = ?1",
                None => "SELECT id, profile_name, api_key, api_secret, api_url, port, is_default FROM api_info WHERE is_default = 1",
            };

            let mut stmt = conn.prepare(query)?;
            let api_info = if let Some(name) = profile_name {
                stmt.query_row(params![name], |row| self.row_to_api_info(row))
            } else {
                stmt.query_row([], |row| self.row_to_api_info(row))
            };

            match api_info {
                Ok(info) => Ok(Some(info)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }
    }

    pub fn get_default_api_info(&self) -> Result<Option<ApiInfo>> {
        self.get_api_info(None)
    }

    fn row_to_api_info(&self, row: &rusqlite::Row) -> rusqlite::Result<ApiInfo> {
        Ok(ApiInfo {
            id: row.get(0)?,
            profile_name: row.get(1)?,
            api_key: row.get(2)?,
            api_secret: row.get(3)?,
            api_url: row.get(4)?,
            port: row.get(5)?,
            is_default: row.get(6)?,
        })
    }

    pub fn list_api_profiles(&self) -> Result<Vec<ApiInfo>> {
        let conn = self.conn.lock().unwrap();

        let has_encrypted_columns: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('api_info') WHERE name='encrypted_api_key'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        if has_encrypted_columns {
            let mut stmt = conn.prepare(
                "SELECT id, profile_name, api_url, port, is_default FROM api_info ORDER BY profile_name"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(ApiInfo {
                    id: row.get(0)?,
                    profile_name: row.get(1)?,
                    api_key: String::new(),
                    api_secret: String::new(),
                    api_url: row.get(2)?,
                    port: row.get(3)?,
                    is_default: row.get(4)?,
                })
            })?;

            rows.collect::<Result<Vec<ApiInfo>, _>>()
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, profile_name, api_key, api_secret, api_url, port, is_default FROM api_info ORDER BY profile_name"
            )?;

            let profiles = stmt
                .query_map([], |row| self.row_to_api_info(row))?
                .collect::<Result<Vec<ApiInfo>, _>>()?;
            Ok(profiles)
        }
    }

    pub fn delete_api_profile(&self, profile_name: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let profile_count: i64 =
            tx.query_row("SELECT COUNT(*) FROM api_info", [], |row| row.get(0))?;

        if profile_count <= 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let is_default: bool = tx.query_row(
            "SELECT is_default FROM api_info WHERE profile_name = ?1",
            params![profile_name],
            |row| row.get(0),
        )?;

        tx.execute(
            "DELETE FROM api_info WHERE profile_name = ?1",
            params![profile_name],
        )?;

        if is_default {
            tx.execute(
                "UPDATE api_info SET is_default = 1 WHERE rowid = (SELECT MIN(rowid) FROM api_info)",
                [],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(
        hash: &str,
        password: &str,
    ) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn verify_pin(&self, pin: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let password_hash_result: rusqlite::Result<String> = conn.query_row(
            "SELECT password_hash FROM app_settings WHERE id = 1",
            [],
            |row| row.get(0),
        );

        match password_hash_result {
            Ok(password_hash) => {
                let result = Self::verify_password(&password_hash, pin).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        )),
                    )
                })?;

                if result {
                    log::info!("PIN verified successfully, saving to cache");
                    self.pin_cache.set_pin(pin.to_string());

                    let salt: String = conn
                        .query_row(
                            "SELECT pin_salt FROM app_settings WHERE id = 1",
                            [],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            error!("Failed to get PIN salt: {}", e);
                            e
                        })?;

                    drop(conn);

                    match self.complete_migration(pin) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to complete migration: {}", e);
                        }
                    }

                    let key = self.derive_encryption_key(pin, &salt).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)),
                        )
                    })?;

                    self.set_current_pin_key(key);
                }

                Ok(result)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn update_password_hash(&self, new_hash: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let salt = SaltString::generate(&mut OsRng).to_string();

        conn.execute(
            "INSERT OR REPLACE INTO app_settings (id, password_hash, pin_salt) VALUES (1, ?1, ?2)",
            params![new_hash, salt],
        )?;

        let mut current_key = self.current_pin_key.lock().unwrap();
        *current_key = None;

        Ok(())
    }

    pub fn update_pin(&self, current_pin: &str, new_pin: &str) -> Result<(), String> {
        if !self
            .verify_pin(current_pin)
            .map_err(|e| format!("Failed to verify current PIN: {}", e))?
        {
            return Err("Current PIN is incorrect".to_string());
        }

        log::info!("Current PIN verified successfully, proceeding with PIN update");

        let api_profiles = self
            .list_api_profiles()
            .map_err(|e| format!("Failed to list API profiles: {}", e))?;

        let mut decrypted_profiles = Vec::new();

        for profile in &api_profiles {
            match self.get_api_info(Some(&profile.profile_name)) {
                Ok(Some(api_info)) => {
                    log::info!(
                        "Successfully retrieved credentials for profile: {}",
                        profile.profile_name
                    );
                    decrypted_profiles.push(api_info);
                }
                _ => {
                    log::warn!(
                        "Failed to get credentials for profile: {}",
                        profile.profile_name
                    );
                }
            }
        }

        let new_hash =
            Self::hash_password(new_pin).map_err(|e| format!("Failed to hash new PIN: {}", e))?;

        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE app_settings SET password_hash = ? WHERE id = 1",
                params![new_hash],
            )
            .map_err(|e| format!("Failed to update PIN hash: {}", e))?;
        }

        {
            let mut current_key = self.current_pin_key.lock().unwrap();
            *current_key = None;
        }

        if !self
            .verify_pin(new_pin)
            .map_err(|e| format!("Failed to verify new PIN: {}", e))?
        {
            return Err("Something went wrong with the new PIN setup".to_string());
        }

        log::info!(
            "Re-saving {} profiles with new PIN",
            decrypted_profiles.len()
        );

        for api_info in decrypted_profiles {
            self.save_api_info(&api_info).map_err(|e| {
                format!(
                    "Failed to save API info for profile '{}': {}",
                    api_info.profile_name, e
                )
            })?;

            log::info!("Re-saved profile: {}", api_info.profile_name);
        }

        log::info!("PIN updated successfully. All applicable API credentials were re-encrypted.");

        Ok(())
    }
    pub fn get_dashboard_preferences(&self, profile_id: i64) -> Result<HashMap<String, DashboardWidgetPref>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT widget_key, visible, position FROM dashboard_preferences WHERE profile_id = ?1"
        )?;
        
        let rows = stmt.query_map([profile_id], |row| {
            Ok(DashboardWidgetPref {
                widget_key: row.get(0)?,
                visible: row.get::<_, i32>(1)? == 1,
                position: row.get(2)?,
            })
        })?;
        
        let mut preferences = HashMap::new();
        for row_result in rows {
            let pref = row_result?;
            preferences.insert(pref.widget_key.clone(), pref);
        }
        
        Ok(preferences)
    }
    
    pub fn save_dashboard_preferences(&self, profile_id: i64, preferences: &[DashboardWidgetPref]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        
        // Clear existing preferences for this profile
        tx.execute(
            "DELETE FROM dashboard_preferences WHERE profile_id = ?1",
            [profile_id],
        )?;
        
        // Insert new preferences
        for pref in preferences {
            tx.execute(
                "INSERT INTO dashboard_preferences (profile_id, widget_key, visible, position) 
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    profile_id,
                    pref.widget_key,
                    if pref.visible { 1 } else { 0 },
                    pref.position
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
    
}
