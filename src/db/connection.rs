use std::{path::{Path, PathBuf}, time::Duration};

use anyhow::{bail, Context, Result};
use rusqlite::{Connection, OpenFlags};

use super::{export, mutations, query, schema};
use super::{ColumnInfo, ColumnSpec, DbObject, QueryResult, RowIdentity, SqlValue, TableData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    Deferred,
    Immediate,
    Exclusive,
}

impl TransactionMode {
    fn sql(self) -> &'static str {
        match self {
            Self::Deferred => "DEFERRED",
            Self::Immediate => "IMMEDIATE",
            Self::Exclusive => "EXCLUSIVE",
        }
    }
}

/// Owns one SQLite connection. Mutations use parameter binding and savepoints;
/// explicit transactions are connection-scoped and never silently auto-committed.
#[derive(Default)]
pub struct DatabaseManager {
    connection: Option<Connection>,
    path: Option<PathBuf>,
    read_only: bool,
}

impl DatabaseManager {
    pub fn is_open(&self) -> bool { self.connection.is_some() }
    pub fn path(&self) -> Option<&Path> { self.path.as_deref() }
    pub fn is_read_only(&self) -> bool { self.read_only }

    pub fn is_in_transaction(&self) -> bool {
        self.connection.as_ref().is_some_and(|connection| !connection.is_autocommit())
    }

    pub fn open(&mut self, path: impl AsRef<Path>, read_only: bool) -> Result<()> {
        if self.is_in_transaction() { bail!("Commit or roll back the active transaction before replacing the database connection"); }
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

    pub fn create(&mut self, path: impl AsRef<Path>) -> Result<()> { self.open(path, false) }

    pub fn open_demo(&mut self) -> Result<()> {
        if self.is_in_transaction() { bail!("Commit or roll back the active transaction before replacing the database connection"); }
        let connection = Connection::open_in_memory().context("Could not create demo database")?;
        Self::configure(&connection, false)?;
        connection.execute_batch(include_str!("../../assets/sample/demo.sql"))?;
        self.connection = Some(connection);
        self.path = None;
        self.read_only = false;
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(connection) = &self.connection
            && !connection.is_autocommit()
            && let Err(error) = connection.execute_batch("ROLLBACK")
        {
            tracing::warn!("Could not roll back transaction while closing database: {error}");
        }
        self.connection = None;
        self.path = None;
        self.read_only = false;
    }

    pub fn sqlite_version(&self) -> Result<String> {
        Ok(self.connection()?.query_row("SELECT sqlite_version()", [], |row| row.get(0))?)
    }

    pub fn list_objects(&self) -> Result<Vec<DbObject>> { schema::list_objects(self.connection()?) }
    pub fn table_columns(&self, table: &str) -> Result<Vec<ColumnInfo>> { schema::table_columns(self.connection()?, table) }
    pub fn table_data(&self, table: &str, limit: usize, offset: usize) -> Result<TableData> {
        schema::table_data(self.connection()?, table, limit, offset)
    }
    pub fn table_ddl(&self, table: &str) -> Result<Option<String>> { schema::table_ddl(self.connection()?, table) }
    pub fn execute(&self, sql: &str) -> Result<QueryResult> { query::execute(self.connection()?, sql) }

    pub fn begin_transaction(&self, mode: TransactionMode) -> Result<()> {
        self.ensure_writable()?;
        let connection = self.connection()?;
        if !connection.is_autocommit() { bail!("A transaction is already active"); }
        connection.execute_batch(&format!("BEGIN {}", mode.sql()))
            .context("Could not begin transaction")
    }

    pub fn commit_transaction(&self) -> Result<()> {
        // Ending a read-only transaction is safe and may be required when BEGIN was
        // issued manually through the Workbench.
        let connection = self.connection()?;
        if connection.is_autocommit() { bail!("There is no active transaction to commit"); }
        connection.execute_batch("COMMIT").context("Could not commit transaction")
    }

    pub fn rollback_transaction(&self) -> Result<()> {
        // Rollback discards pending state and is also valid for a read-only snapshot.
        let connection = self.connection()?;
        if connection.is_autocommit() { bail!("There is no active transaction to roll back"); }
        connection.execute_batch("ROLLBACK").context("Could not roll back transaction")
    }

    pub fn insert_row(&self, table: &str, values: &[(String, SqlValue)]) -> Result<usize> {
        self.ensure_writable()?;
        mutations::insert_row(self.connection()?, table, values)
    }

    pub fn update_row(&self, table: &str, identity: &RowIdentity, changes: &[(String, SqlValue)]) -> Result<usize> {
        self.ensure_writable()?;
        mutations::update_row(self.connection()?, table, identity, changes)
    }

    pub fn delete_row(&self, table: &str, identity: &RowIdentity) -> Result<usize> {
        self.ensure_writable()?;
        mutations::delete_row(self.connection()?, table, identity)
    }

    pub fn create_table(&self, name: &str, columns: &[ColumnSpec]) -> Result<()> {
        self.ensure_writable()?;
        mutations::create_table(self.connection()?, name, columns)
    }

    pub fn rename_table(&self, table: &str, new_name: &str) -> Result<()> {
        self.ensure_writable()?;
        mutations::rename_table(self.connection()?, table, new_name)
    }

    pub fn add_column(&self, table: &str, name: &str, data_type: &str, not_null: bool, default_value: Option<&SqlValue>) -> Result<()> {
        self.ensure_writable()?;
        mutations::add_column(self.connection()?, table, name, data_type, not_null, default_value)
    }

    pub fn rename_column(&self, table: &str, old_name: &str, new_name: &str) -> Result<()> {
        self.ensure_writable()?;
        mutations::rename_column(self.connection()?, table, old_name, new_name)
    }

    pub fn drop_column(&self, table: &str, column: &str) -> Result<()> {
        self.ensure_writable()?;
        mutations::drop_column(self.connection()?, table, column)
    }

    pub fn drop_table(&self, table: &str) -> Result<()> {
        self.ensure_writable()?;
        mutations::drop_table(self.connection()?, table)
    }

    pub fn integrity_check(&self) -> Result<Vec<String>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("PRAGMA integrity_check")?;
        let values = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut messages = Vec::new();
        for value in values { messages.push(value?); }
        Ok(messages)
    }

    pub fn export_csv(&self, result: &QueryResult, path: impl AsRef<Path>) -> Result<()> {
        export::write_csv(result, path)
    }

    fn connection(&self) -> Result<&Connection> {
        self.connection.as_ref().context("Open a SQLite database first")
    }

    fn ensure_writable(&self) -> Result<()> {
        if self.read_only { bail!("This database was opened read-only"); }
        if !self.is_open() { bail!("Open a SQLite database first"); }
        Ok(())
    }

    fn configure(connection: &Connection, read_only: bool) -> Result<()> {
        connection.busy_timeout(Duration::from_millis(5_000))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        if read_only { connection.pragma_update(None, "query_only", "ON")?; }
        if !read_only
            && connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get::<_, String>(0)).is_err()
        {
            tracing::debug!("SQLite WAL mode request was not accepted");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::Value;

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
    fn read_only_session_rejects_data_and_schema_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("readonly.sqlite");
        {
            let connection = Connection::open(&path).unwrap();
            connection.execute_batch("CREATE TABLE sample(value TEXT);").unwrap();
        }
        let mut db = DatabaseManager::default();
        db.open(&path, true).unwrap();
        assert!(db.execute("INSERT INTO sample(value) VALUES ('blocked')").is_err());
        assert!(db.insert_row("sample", &[("value".into(), Value::Text("blocked".into()))]).is_err());
        assert!(db.create_table("blocked_table", &[ColumnSpec { name: "id".into(), data_type: "INTEGER".into(), not_null: false, primary_key: false }]).is_err());
        assert_eq!(db.execute("SELECT COUNT(*) FROM sample").unwrap().rows[0][0], "0");
    }

    #[test]
    fn read_only_workbench_transaction_can_be_ended() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("readonly-transaction.sqlite");
        Connection::open(&path).unwrap().execute_batch("CREATE TABLE sample(value TEXT);").unwrap();
        let mut db = DatabaseManager::default();
        db.open(&path, true).unwrap();
        db.execute("BEGIN DEFERRED").unwrap();
        assert!(db.is_in_transaction());
        db.commit_transaction().unwrap();
        assert!(!db.is_in_transaction());
    }

    #[test]
    fn explicit_transaction_commit_and_rollback_work() {
        let mut db = DatabaseManager::default();
        db.open_demo().unwrap();
        db.begin_transaction(TransactionMode::Immediate).unwrap();
        assert!(db.is_in_transaction());
        db.execute("INSERT INTO customers(name, email, country) VALUES ('Commit Test', 'commit@example.test', 'Test Country')").unwrap();
        db.commit_transaction().unwrap();
        assert!(!db.is_in_transaction());
        assert_eq!(db.execute("SELECT COUNT(*) FROM customers WHERE email='commit@example.test'").unwrap().rows[0][0], "1");
        db.begin_transaction(TransactionMode::Deferred).unwrap();
        db.execute("INSERT INTO customers(name, email, country) VALUES ('Rollback Test', 'rollback@example.test', 'Test Country')").unwrap();
        db.rollback_transaction().unwrap();
        assert_eq!(db.execute("SELECT COUNT(*) FROM customers WHERE email='rollback@example.test'").unwrap().rows[0][0], "0");
    }

    #[test]
    fn failed_guided_write_keeps_outer_transaction_open() {
        let mut db = DatabaseManager::default();
        db.open_demo().unwrap();
        db.create_table("unique_sample", &[
            ColumnSpec { name: "id".into(), data_type: "INTEGER".into(), not_null: true, primary_key: true },
            ColumnSpec { name: "value".into(), data_type: "TEXT".into(), not_null: true, primary_key: false },
        ]).unwrap();
        db.begin_transaction(TransactionMode::Immediate).unwrap();
        db.insert_row("unique_sample", &[("id".into(), Value::Integer(1)), ("value".into(), Value::Text("kept".into()))]).unwrap();
        db.create_table("unique_guard", &[
            ColumnSpec { name: "value".into(), data_type: "TEXT".into(), not_null: true, primary_key: true },
        ]).unwrap();
        db.insert_row("unique_guard", &[("value".into(), Value::Text("duplicate".into()))]).unwrap();
        let failed = db.insert_row("unique_guard", &[("value".into(), Value::Text("duplicate".into()))]);
        assert!(failed.is_err());
        assert!(db.is_in_transaction());
        assert_eq!(db.execute("SELECT COUNT(*) FROM unique_sample").unwrap().rows[0][0], "1");
        db.commit_transaction().unwrap();
    }

    #[test]
    fn active_transaction_blocks_database_replacement() {
        let mut db = DatabaseManager::default();
        db.open_demo().unwrap();
        db.begin_transaction(TransactionMode::Deferred).unwrap();
        assert!(db.open_demo().is_err());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("replacement.sqlite");
        assert!(db.open(&path, false).is_err());
        assert!(db.is_in_transaction());
        db.rollback_transaction().unwrap();
    }

    #[test]
    fn crud_and_schema_changes_are_available() {
        let mut db = DatabaseManager::default();
        db.open_demo().unwrap();
        db.create_table("crud_sample", &[ColumnSpec { name: "id".into(), data_type: "INTEGER".into(), not_null: true, primary_key: true }, ColumnSpec { name: "title".into(), data_type: "TEXT".into(), not_null: false, primary_key: false }]).unwrap();
        db.insert_row("crud_sample", &[("id".into(), Value::Integer(1)), ("title".into(), Value::Text("first".into()))]).unwrap();
        db.update_row("crud_sample", &RowIdentity::PrimaryKey(vec![("id".into(), Value::Integer(1))]), &[("title".into(), Value::Text("changed".into()))]).unwrap();
        assert_eq!(db.table_data("crud_sample", 10, 0).unwrap().rows[0][1], "changed");
        db.add_column("crud_sample", "enabled", "INTEGER", false, Some(&Value::Integer(1))).unwrap();
        db.rename_column("crud_sample", "enabled", "is_enabled").unwrap();
        db.rename_table("crud_sample", "crud_renamed").unwrap();
        db.drop_column("crud_renamed", "is_enabled").unwrap();
        db.drop_table("crud_renamed").unwrap();
    }
}
