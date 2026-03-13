use crate::error::Result;
use rusqlite::Connection;

const SCHEMA_VERSION: i32 = 3;

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            parent_id TEXT REFERENCES groups(id),
            sort_order INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
            type TEXT NOT NULL CHECK(type IN ('account', 'api_key', 'env_var')),
            title TEXT NOT NULL,
            icon TEXT,
            is_favorite INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS account_items (
            item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            username TEXT NOT NULL,
            password_encrypted BLOB NOT NULL,
            password_nonce BLOB NOT NULL,
            website TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS api_key_items (
            item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            key_name TEXT NOT NULL,
            key_value_encrypted BLOB NOT NULL,
            key_value_nonce BLOB NOT NULL,
            endpoint TEXT,
            auth_method TEXT,
            notes TEXT,
            rotation_date INTEGER
        );

        CREATE TABLE IF NOT EXISTS env_var_items (
            item_id TEXT REFERENCES items(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value_encrypted BLOB NOT NULL,
            value_nonce BLOB NOT NULL,
            sort_order INTEGER DEFAULT 0,
            notes TEXT,
            PRIMARY KEY (item_id, key)
        );

        CREATE TABLE IF NOT EXISTS vault_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_items_group ON items(group_id);
        CREATE INDEX IF NOT EXISTS idx_items_type ON items(type);
        CREATE INDEX IF NOT EXISTS idx_items_favorite ON items(is_favorite);
        "#,
    )
    .map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;

    set_schema_version(conn, SCHEMA_VERSION)?;

    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version = get_schema_version(conn)?;

    // Already at latest version, nothing to migrate
    if version >= SCHEMA_VERSION {
        return Ok(());
    }

    // Migration from version 0 or 1 to 2
    if version < 2 {
        // Check if auth_method column exists
        let has_auth_method: std::result::Result<i32, _> = conn.query_row(
            "SELECT 1 FROM pragma_table_info('api_key_items') WHERE name='auth_method'",
            [],
            |_| Ok(1),
        );
        if has_auth_method.is_err() {
            let _ = conn.execute("ALTER TABLE api_key_items ADD COLUMN auth_method TEXT", []);
        }

        let has_rotation_date: std::result::Result<i32, _> = conn.query_row(
            "SELECT 1 FROM pragma_table_info('api_key_items') WHERE name='rotation_date'",
            [],
            |_| Ok(1),
        );
        if has_rotation_date.is_err() {
            let _ = conn.execute(
                "ALTER TABLE api_key_items ADD COLUMN rotation_date INTEGER",
                [],
            );
        }
    }

    // Migration to version 3
    if version < 3 {
        let has_notes: std::result::Result<i32, _> = conn.query_row(
            "SELECT 1 FROM pragma_table_info('env_var_items') WHERE name='notes'",
            [],
            |_| Ok(1),
        );
        if has_notes.is_err() {
            let _ = conn.execute("ALTER TABLE env_var_items ADD COLUMN notes TEXT", []);
        }
    }

    set_schema_version(conn, SCHEMA_VERSION)?;
    Ok(())
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('schema_version', ?)",
        [version.to_string()],
    )
    .map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version: std::result::Result<i32, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    );

    match version {
        Ok(v) => Ok(v),
        Err(_) => Ok(0),
    }
}

pub fn insert_builtin_groups(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        INSERT OR IGNORE INTO groups (id, name, icon, sort_order, created_at, updated_at)
        VALUES 
            ('built-in-accounts', 'Accounts', '🔑', 0, strftime('%s', 'now'), strftime('%s', 'now')),
            ('built-in-api-keys', 'API Keys', '🔧', 1, strftime('%s', 'now'), strftime('%s', 'now')),
            ('built-in-env-vars', 'Environment Variables', '📦', 2, strftime('%s', 'now'), strftime('%s', 'now'));
        "#,
    ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}
