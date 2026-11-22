use anyhow::Result;
use rusqlite::{types::{Value, ValueRef}, Connection};

use super::mutations::RowIdentity;

#[derive(Debug, Clone)]
pub struct DbObject {
    pub kind: String,
    pub name: String,
    pub table_name: String,
    pub sql: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub ordinal: i64,
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key_order: i64,
    /// SQLite table_xinfo hidden flag: 0 normal, 1 virtual-table hidden, 2/3 generated.
    pub hidden: i64,
}

#[derive(Debug, Clone, Default)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Owned SQLite values retained for typed editing; display strings are never used for writes.
    pub raw_rows: Vec<Vec<Value>>,
    /// `None` for views or tables where SQLite does not expose a safe row identity.
    pub row_identities: Vec<Option<RowIdentity>>,
    pub offset: usize,
    pub limit: usize,
}

pub(crate) fn list_objects(connection: &Connection) -> Result<Vec<DbObject>> {
    let mut statement = connection.prepare(
        "SELECT type, name, tbl_name, sql
         FROM sqlite_schema
         WHERE name NOT LIKE 'sqlite_%'
           AND type IN ('table', 'view', 'index', 'trigger')
         ORDER BY CASE type
             WHEN 'table' THEN 0 WHEN 'view' THEN 1
             WHEN 'index' THEN 2 ELSE 3 END, name COLLATE NOCASE",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(DbObject {
            kind: row.get(0)?,
            name: row.get(1)?,
            table_name: row.get(2)?,
            sql: row.get(3)?,
        })
    })?;

    let mut objects = Vec::new();
    for row in rows {
        objects.push(row?);
    }
    Ok(objects)
}

pub(crate) fn table_columns(connection: &Connection, table: &str) -> Result<Vec<ColumnInfo>> {
    let pragma = format!("PRAGMA table_xinfo({})", quote_sql_string(table));
    let mut statement = connection.prepare(&pragma)?;
    let rows = statement.query_map([], |row| {
        Ok(ColumnInfo {
            ordinal: row.get(0)?,
            name: row.get(1)?,
            data_type: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            not_null: row.get::<_, i64>(3)? != 0,
            default_value: row.get(4)?,
            primary_key_order: row.get(5)?,
            hidden: row.get(6)?,
        })
    })?;

    let mut columns = Vec::new();
    for row in rows {
        let column = row?;
        // Hidden virtual-table implementation columns are not part of SELECT *.
        if column.hidden != 1 {
            columns.push(column);
        }
    }
    Ok(columns)
}

pub(crate) fn table_data(
    connection: &Connection,
    table: &str,
    limit: usize,
    offset: usize,
) -> Result<TableData> {
    let column_info = table_columns(connection, table)?;
    let primary_key = {
        let mut columns = column_info
            .iter()
            .filter(|column| column.primary_key_order > 0)
            .collect::<Vec<_>>();
        columns.sort_by_key(|column| column.primary_key_order);
        columns
    };
    let table_sql = connection
        .query_row(
            "SELECT COALESCE(sql, '') FROM sqlite_schema WHERE name = ?1 AND type = 'table'",
            [table],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();
    let is_rowid_table = !table_sql.is_empty()
        && !table_sql.trim_start().to_ascii_uppercase().starts_with("CREATE VIRTUAL TABLE");

    if !primary_key.is_empty() {
        let sql = format!(
            "SELECT * FROM {} LIMIT ?1 OFFSET ?2",
            quote_identifier(table)
        );
        let mut statement = connection.prepare(&sql)?;
        let columns = statement.column_names().map(str::to_owned).collect::<Vec<_>>();
        let primary_key_indices = primary_key.iter().map(|column| {
            let index = columns.iter().position(|name| name == &column.name)
                .ok_or_else(|| anyhow::anyhow!("Primary-key column {} is not present in the table projection", column.name))?;
            Ok((column.name.clone(), index))
        }).collect::<Result<Vec<_>>>()?;
        let count = statement.column_count();
        let mut cursor = statement.query((limit as i64, offset as i64))?;
        let mut data = TableData { columns, offset, limit, ..Default::default() };
        while let Some(row) = cursor.next()? {
            let raw = owned_row(row, count)?;
            let display = raw.iter().map(display_owned_value).collect::<Vec<_>>();
            let identity = primary_key_indices
                .iter()
                .map(|(name, index)| (name.clone(), raw[*index].clone()))
                .collect();
            data.rows.push(display);
            data.raw_rows.push(raw);
            data.row_identities.push(Some(RowIdentity::PrimaryKey(identity)));
        }
        return Ok(data);
    }

    if is_rowid_table {
        let names = column_info.iter().map(|column| column.name.to_lowercase()).collect::<Vec<_>>();
        let rowid_alias = ["rowid", "_rowid_", "oid"]
            .into_iter()
            .find(|alias| !names.iter().any(|name| name == alias));
        if let Some(alias) = rowid_alias {
            let sql = format!(
                "SELECT {}, * FROM {} LIMIT ?1 OFFSET ?2",
                quote_identifier(alias),
                quote_identifier(table)
            );
            if let Ok(mut statement) = connection.prepare(&sql) {
                let columns = statement.column_names().skip(1).map(str::to_owned).collect::<Vec<_>>();
                let count = statement.column_count();
                let mut cursor = statement.query((limit as i64, offset as i64))?;
                let mut data = TableData { columns, offset, limit, ..Default::default() };
                while let Some(row) = cursor.next()? {
                    let rowid: i64 = row.get(0)?;
                    let raw = owned_row_from(row, 1, count)?;
                    let display = raw.iter().map(display_owned_value).collect::<Vec<_>>();
                    data.rows.push(display);
                    data.raw_rows.push(raw);
                    data.row_identities.push(Some(RowIdentity::RowId { alias: alias.to_owned(), value: rowid }));
                }
                return Ok(data);
            }
        }
    }

    // Read-only views and WITHOUT ROWID tables without a declared PK can still be browsed.
    let sql = format!("SELECT * FROM {} LIMIT ?1 OFFSET ?2", quote_identifier(table));
    let mut statement = connection.prepare(&sql)?;
    let columns = statement.column_names().map(str::to_owned).collect::<Vec<_>>();
    let count = statement.column_count();
    let mut cursor = statement.query((limit as i64, offset as i64))?;
    let mut data = TableData { columns, offset, limit, ..Default::default() };
    while let Some(row) = cursor.next()? {
        let raw = owned_row(row, count)?;
        data.rows.push(raw.iter().map(display_owned_value).collect());
        data.raw_rows.push(raw);
        data.row_identities.push(None);
    }
    Ok(data)
}

pub(crate) fn table_ddl(connection: &Connection, table: &str) -> Result<Option<String>> {
    Ok(connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE name = ?1 AND type IN ('table', 'view')",
            [table],
            |row| row.get::<_, String>(0),
        )
        .ok())
}

fn owned_row(row: &rusqlite::Row<'_>, count: usize) -> rusqlite::Result<Vec<Value>> {
    owned_row_from(row, 0, count)
}

fn owned_row_from(row: &rusqlite::Row<'_>, start: usize, count: usize) -> rusqlite::Result<Vec<Value>> {
    (start..count).map(|index| {
        let value = row.get_ref(index)?;
        Ok(match value {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(value) => Value::Integer(value),
            ValueRef::Real(value) => Value::Real(value),
            ValueRef::Text(value) => Value::Text(String::from_utf8_lossy(value).into_owned()),
            ValueRef::Blob(value) => Value::Blob(value.to_vec()),
        })
    }).collect()
}

fn display_owned_value(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_owned(),
        Value::Integer(value) => value.to_string(),
        Value::Real(value) => value.to_string(),
        Value::Text(value) => {
            const MAX_TEXT_CHARS: usize = 16_384;
            let mut chars = value.chars();
            let mut rendered = chars.by_ref().take(MAX_TEXT_CHARS).collect::<String>();
            if chars.next().is_some() { rendered.push_str("… [cell truncated]"); }
            rendered
        }
        Value::Blob(value) => format!("[BLOB · {} bytes]", value.len()),
    }
}

pub(crate) fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn quote_sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_quoted_safely() {
        assert_eq!(quote_identifier("odd\"table"), "\"odd\"\"table\"");
    }

    #[test]
    fn table_metadata_and_data_are_available() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE sample(id INTEGER PRIMARY KEY, label TEXT NOT NULL);
             INSERT INTO sample(label) VALUES ('hello');",
        ).unwrap();
        let columns = table_columns(&connection, "sample").unwrap();
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[1].name, "label");
        let data = table_data(&connection, "sample", 20, 0).unwrap();
        assert_eq!(data.rows.len(), 1);
        assert_eq!(data.rows[0][1], "hello");
        assert_eq!(data.row_identities.len(), 1);
        assert!(data.row_identities[0].is_some());
    }

    #[test]
    fn no_primary_key_table_uses_hidden_rowid_when_available() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE sample(value TEXT); INSERT INTO sample VALUES ('hello');").unwrap();
        let data = table_data(&connection, "sample", 20, 0).unwrap();
        assert_eq!(data.columns, vec!["value"]);
        assert!(matches!(data.row_identities[0], Some(RowIdentity::RowId { .. })));
    }

    #[test]
    fn generated_columns_are_loaded_and_primary_keys_are_mapped_by_name() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE sample(derived INTEGER GENERATED ALWAYS AS (base * 2) STORED, base INTEGER, code TEXT, PRIMARY KEY(base, code)); INSERT INTO sample(base, code) VALUES (4, 'x');").unwrap();
        let columns = table_columns(&connection, "sample").unwrap();
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].hidden, 3);
        let data = table_data(&connection, "sample", 10, 0).unwrap();
        assert_eq!(data.rows[0], vec!["8", "4", "x"]);
        assert_eq!(data.row_identities[0], Some(RowIdentity::PrimaryKey(vec![
            ("base".into(), Value::Integer(4)),
            ("code".into(), Value::Text("x".into())),
        ])));
    }
}
