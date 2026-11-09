use anyhow::Result;
use rusqlite::Connection;

use super::value::display_value;

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
}

#[derive(Debug, Clone, Default)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
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
    let pragma = format!("PRAGMA table_info({})", quote_sql_string(table));
    let mut statement = connection.prepare(&pragma)?;
    let rows = statement.query_map([], |row| {
        Ok(ColumnInfo {
            ordinal: row.get(0)?,
            name: row.get(1)?,
            data_type: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            not_null: row.get::<_, i64>(3)? != 0,
            default_value: row.get(4)?,
            primary_key_order: row.get(5)?,
        })
    })?;

    let mut columns = Vec::new();
    for row in rows {
        columns.push(row?);
    }
    Ok(columns)
}

pub(crate) fn table_data(
    connection: &Connection,
    table: &str,
    limit: usize,
    offset: usize,
) -> Result<TableData> {
    let sql = format!(
        "SELECT * FROM {} LIMIT ?1 OFFSET ?2",
        quote_identifier(table)
    );
    let mut statement = connection.prepare(&sql)?;
    let columns = statement.column_names().map(str::to_owned).collect();
    let column_count = statement.column_count();
    let mut rows_cursor = statement.query((limit as i64, offset as i64))?;
    let mut rows = Vec::new();

    while let Some(row) = rows_cursor.next()? {
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            values.push(display_value(row.get_ref(index)?));
        }
        rows.push(values);
    }

    Ok(TableData { columns, rows, offset, limit })
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
    }
}
