use std::{path::{Path, PathBuf}, time::Duration};

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};

use super::{export, query, schema};
use super::{ColumnInfo, DbObject, QueryResult, TableData};

/// Owns one SQLite connection. The prototype intentionally keeps database access on
/// the UI thread; the production roadmap moves this owner into a cancellable DB actor.
#[derive(Default)]
pub struct DatabaseManager {
    connection: Option<Connection>,
    path: Option<PathBuf>,
    read_only: bool,
}

impl DatabaseManager {
    pub fn is_open(&self) -> bool {
        self.connection.is_some()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn open(&mut self, path: impl AsRef<Path>, read_only: bool) -> Result<()> {
        let path = path.as_ref();
        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        };

        let connection = Connection::open_with_flags(path, flags)
            .with_context(|| format!("Could not open database: {}", path.display()))?;
        Self::configure(&connection, read_only)?;

        self.connection = Some(connection);
        self.path = Some(path.to_path_buf());
        self.read_only = read_only;
        Ok(())
    }

    pub fn create(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.open(path, false)
    }

    pub fn open_demo(&mut self) -> Result<()> {
        let connection = Connection::open_in_memory().context("Could not create demo database")?;
        Self::configure(&connection, false)?;
        connection.execute_batch(include_str!("../../assets/sample/demo.sql"))?;
        self.connection = Some(connection);
        self.path = None;
        self.read_only = false;
        Ok(())
    }

    pub fn close(&mut self) {
        self.connection = None;
        self.path = None;
        self.read_only = false;
    }

    pub fn sqlite_version(&self) -> Result<String> {
        let connection = self.connection()?;
        Ok(connection.query_row("SELECT sqlite_version()", [], |row| row.get(0))?)
    }

    pub fn list_objects(&self) -> Result<Vec<DbObject>> {
        schema::list_objects(self.connection()?)
    }

    pub fn table_columns(&self, table: &str) -> Result<Vec<ColumnInfo>> {
        schema::table_columns(self.connection()?, table)
    }

    pub fn table_data(&self, table: &str, limit: usize, offset: usize) -> Result<TableData> {
        schema::table_data(self.connection()?, table, limit, offset)
    }

    pub fn table_ddl(&self, table: &str) -> Result<Option<String>> {
        schema::table_ddl(self.connection()?, table)
    }

    pub fn execute(&self, sql: &str) -> Result<QueryResult> {
        query::execute(self.connection()?, sql)
    }

    pub fn integrity_check(&self) -> Result<Vec<String>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("PRAGMA integrity_check")?;
        let values = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut messages = Vec::new();
        for value in values {
            messages.push(value?);
        }
        Ok(messages)
    }

    pub fn export_csv(&self, result: &QueryResult, path: impl AsRef<Path>) -> Result<()> {
        export::write_csv(result, path)
    }

    fn connection(&self) -> Result<&Connection> {
        self.connection.as_ref().context("Open a SQLite database first")
    }

    fn configure(connection: &Connection, read_only: bool) -> Result<()> {
        connection.busy_timeout(Duration::from_millis(5_000))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        if read_only {
            // Add a connection-level guard on top of SQLITE_OPEN_READ_ONLY.
            connection.pragma_update(None, "query_only", "ON")?;
        }
        if !read_only {
            // WAL improves reader/writer coexistence while remaining a local file-based mode.
            // SQLite may reject WAL for special filesystems; failure should not block opening.
            if connection
                .query_row("PRAGMA journal_mode = WAL", [], |row| row.get::<_, String>(0))
                .is_err()
            {
                tracing::debug!("SQLite WAL mode request was not accepted");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_demo_and_lists_tables() {
        let mut db = DatabaseManager::default();
        db.open_demo().expect("demo database should open");
        let objects = db.list_objects().expect("schema should be readable");
        assert!(objects.iter().any(|object| object.name == "customers"));
        assert!(objects.iter().any(|object| object.name == "orders"));
    }

    #[test]
    fn executes_select_and_reports_rows() {
        let mut db = DatabaseManager::default();
        db.open_demo().unwrap();
        let result = db.execute("SELECT name FROM customers ORDER BY id").unwrap();
        assert_eq!(result.columns, vec!["name"]);
        assert_eq!(result.rows.len(), 4);
        assert_eq!(result.rows[0][0], "Avery Stone");
    }

    #[test]
    fn read_only_session_rejects_data_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("readonly.sqlite");
        {
            let connection = Connection::open(&path).unwrap();
            connection.execute_batch("CREATE TABLE sample(value TEXT);").unwrap();
        }

        let mut db = DatabaseManager::default();
        db.open(&path, true).unwrap();
        assert!(db.execute("INSERT INTO sample(value) VALUES ('blocked')").is_err());
        assert_eq!(db.execute("SELECT COUNT(*) FROM sample").unwrap().rows[0][0], "0");
    }
}
