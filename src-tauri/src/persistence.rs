use rusqlite::{Connection, params, Result};
use std::path::PathBuf;
use crate::models::kv::RemoteConnection;

const DB_VERSION: i32 = 1;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self> {
      if let Err(_e) = std::fs::create_dir_all(app_data_dir) {
          return Err(rusqlite::Error::ExecuteReturnedResults);
      }

      let db_path = app_data_dir.join("kv_explorer.db");
      let conn = Connection::open(db_path)?;

      Self::initialize_database(&conn)?;

      Ok(Self { conn })
    }

    fn initialize_database(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                last_used INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS remote_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL UNIQUE,
                api_token TEXT NOT NULL,
                last_used INTEGER NOT NULL
            )",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = 'schema_version'")?;
        let version: Result<String> = stmt.query_row([], |row| row.get(0));

        match version {
            Ok(v) => {
                let current_version: i32 = v.parse().unwrap_or(0);
                if current_version < DB_VERSION {
                    conn.execute(
                        "UPDATE app_settings SET value = ? WHERE key = 'schema_version'",
                        params![DB_VERSION.to_string()],
                    )?;
                }
            }
            Err(_) => {
                conn.execute(
                    "INSERT INTO app_settings (key, value) VALUES ('schema_version', ?)",
                    params![DB_VERSION.to_string()],
                )?;
            }
        }

        Ok(())
    }

    pub fn save_folder(&self, path: &str, name: &str) -> Result<i64> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO folders (path, name, last_used) VALUES (?, ?, ?)",
            params![path, name, timestamp],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_folders(&self) -> Result<Vec<(i64, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, name FROM folders ORDER BY last_used DESC"
        )?;

        let folder_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut folders = Vec::new();
        for folder in folder_iter {
            folders.push(folder?);
        }

        Ok(folders)
    }

    pub fn update_folder_timestamp(&self, path: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            "UPDATE folders SET last_used = ? WHERE path = ?",
            params![timestamp, path],
        )?;

        Ok(())
    }

    pub fn remove_folder(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM folders WHERE id = ?",
            params![id],
        )?;

        Ok(())
    }

    pub fn save_remote_connection(&self, account_id: &str, api_token: &str) -> Result<i64> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO remote_connections (account_id, api_token, last_used) VALUES (?, ?, ?)",
            params![account_id, api_token, timestamp],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_remote_connections(&self) -> Result<Vec<RemoteConnection>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, api_token FROM remote_connections ORDER BY last_used DESC"
        )?;

        let connection_iter = stmt.query_map([], |row| {
            Ok(RemoteConnection {
                account_id: row.get::<_, String>(0)?,
                api_token: row.get::<_, String>(1)?,
            })
        })?;

        let mut connections = Vec::new();
        for connection in connection_iter {
            connections.push(connection?);
        }

        Ok(connections)
    }

    pub fn update_connection_timestamp(&self, account_id: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();

        self.conn.execute(
            "UPDATE remote_connections SET last_used = ? WHERE account_id = ?",
            params![timestamp, account_id],
        )?;

        Ok(())
    }

    pub fn remove_all_connections(&self) -> Result<()> {
        self.conn.execute("DELETE FROM remote_connections", [])?;
        Ok(())
    }
}