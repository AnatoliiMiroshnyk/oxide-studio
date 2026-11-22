use anyhow::{bail, ensure, Context, Result};
use rusqlite::{params_from_iter, types::Value, Connection};

use super::schema::quote_identifier;

/// Stable identity for a row. Primary keys are preferred; rowid is used only when
/// SQLite exposes an unshadowed hidden rowid alias for an ordinary rowid table.
#[derive(Debug, Clone, PartialEq)]
pub enum RowIdentity {
    PrimaryKey(Vec<(String, Value)>),
    RowId { alias: String, value: i64 },
}

#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub primary_key: bool,
}

pub(crate) fn insert_row(
    connection: &Connection,
    table: &str,
    values: &[(String, Value)],
) -> Result<usize> {
    if values.is_empty() {
        return in_savepoint(connection, || Ok(connection.execute(
            &format!("INSERT INTO {} DEFAULT VALUES", quote_identifier(table)),
            [],
        )?)).context("Could not insert default row");
    }
    let columns = values
        .iter()
        .map(|(name, _)| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = (1..=values.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO {} ({columns}) VALUES ({placeholders})",
        quote_identifier(table)
    );
    let bound = values.iter().map(|(_, value)| value.clone()).collect::<Vec<_>>();
    in_savepoint(connection, || Ok(connection.execute(&sql, params_from_iter(bound.iter()))?))
        .context("Could not insert row")
}

pub(crate) fn update_row(
    connection: &Connection,
    table: &str,
    identity: &RowIdentity,
    changes: &[(String, Value)],
) -> Result<usize> {
    ensure!(!changes.is_empty(), "No edited values were provided");
    let assignments = changes
        .iter()
        .enumerate()
        .map(|(index, (name, _))| format!("{} = ?{}", quote_identifier(name), index + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let (predicate, keys) = identity_predicate(identity, changes.len() + 1);
    let sql = format!(
        "UPDATE {} SET {assignments} WHERE {predicate}",
        quote_identifier(table)
    );
    let mut bound = changes.iter().map(|(_, value)| value.clone()).collect::<Vec<_>>();
    bound.extend(keys);
    in_savepoint(connection, || {
        let affected = connection.execute(&sql, params_from_iter(bound.iter()))?;
        ensure!(affected <= 1, "Row identity matched more than one row; update was refused");
        Ok(affected)
    })
}

pub(crate) fn delete_row(
    connection: &Connection,
    table: &str,
    identity: &RowIdentity,
) -> Result<usize> {
    let (predicate, keys) = identity_predicate(identity, 1);
    let sql = format!("DELETE FROM {} WHERE {predicate}", quote_identifier(table));
    in_savepoint(connection, || {
        let affected = connection.execute(&sql, params_from_iter(keys.iter()))?;
        ensure!(affected <= 1, "Row identity matched more than one row; delete was refused");
        Ok(affected)
    })
}

pub(crate) fn create_table(connection: &Connection, name: &str, columns: &[ColumnSpec]) -> Result<()> {
    ensure!(!name.trim().is_empty(), "Table name cannot be empty");
    ensure!(!columns.is_empty(), "A table must have at least one column");
    let mut definitions = Vec::with_capacity(columns.len() + 1);
    let mut primary_keys = Vec::new();
    for column in columns {
        ensure!(!column.name.trim().is_empty(), "Column names cannot be empty");
        validate_type_name(&column.data_type)?;
        let mut definition = format!(
            "{} {}",
            quote_identifier(&column.name),
            column.data_type.trim()
        );
        if column.not_null {
            definition.push_str(" NOT NULL");
        }
        definitions.push(definition);
        if column.primary_key {
            primary_keys.push(quote_identifier(&column.name));
        }
    }
    if !primary_keys.is_empty() {
        definitions.push(format!("PRIMARY KEY ({})", primary_keys.join(", ")));
    }
    let sql = format!(
        "CREATE TABLE {} ({})",
        quote_identifier(name),
        definitions.join(", ")
    );
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) })
        .context("Could not create table")
}

pub(crate) fn rename_table(connection: &Connection, table: &str, new_name: &str) -> Result<()> {
    ensure!(!new_name.trim().is_empty(), "New table name cannot be empty");
    let sql = format!(
        "ALTER TABLE {} RENAME TO {}",
        quote_identifier(table),
        quote_identifier(new_name)
    );
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) }).context("Could not rename table")
}

pub(crate) fn add_column(
    connection: &Connection,
    table: &str,
    name: &str,
    data_type: &str,
    not_null: bool,
    default_value: Option<&Value>,
) -> Result<()> {
    ensure!(!name.trim().is_empty(), "Column name cannot be empty");
    validate_type_name(data_type)?;
    let mut definition = format!("{} {}", quote_identifier(name), data_type.trim());
    if not_null {
        definition.push_str(" NOT NULL");
    }
    if let Some(value) = default_value {
        definition.push_str(" DEFAULT ");
        definition.push_str(&sql_literal(value)?);
    }
    let sql = format!(
        "ALTER TABLE {} ADD COLUMN {definition}",
        quote_identifier(table)
    );
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) }).context("Could not add column")
}

pub(crate) fn rename_column(
    connection: &Connection,
    table: &str,
    old_name: &str,
    new_name: &str,
) -> Result<()> {
    ensure!(!new_name.trim().is_empty(), "New column name cannot be empty");
    let sql = format!(
        "ALTER TABLE {} RENAME COLUMN {} TO {}",
        quote_identifier(table),
        quote_identifier(old_name),
        quote_identifier(new_name)
    );
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) }).context("Could not rename column")
}

pub(crate) fn drop_column(connection: &Connection, table: &str, column: &str) -> Result<()> {
    let sql = format!(
        "ALTER TABLE {} DROP COLUMN {}",
        quote_identifier(table),
        quote_identifier(column)
    );
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) }).context("Could not drop column")
}

pub(crate) fn drop_table(connection: &Connection, table: &str) -> Result<()> {
    let sql = format!("DROP TABLE {}", quote_identifier(table));
    in_savepoint(connection, || { connection.execute_batch(&sql)?; Ok(()) }).context("Could not drop table")
}

fn identity_predicate(identity: &RowIdentity, start_index: usize) -> (String, Vec<Value>) {
    match identity {
        RowIdentity::PrimaryKey(keys) => {
            let predicate = keys
                .iter()
                .enumerate()
                .map(|(index, (name, _))| {
                    format!("{} IS ?{}", quote_identifier(name), start_index + index)
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            (predicate, keys.iter().map(|(_, value)| value.clone()).collect())
        }
        RowIdentity::RowId { alias, value } => (
            format!("{} = ?{start_index}", quote_identifier(alias)),
            vec![Value::Integer(*value)],
        ),
    }
}

fn validate_type_name(data_type: &str) -> Result<()> {
    let value = data_type.trim().to_ascii_uppercase();
    ensure!(!value.is_empty(), "SQLite column type cannot be empty");
    ensure!(value.len() <= 64, "Column type is too long");

    let (base, dimensions) = match value.split_once('(') {
        Some((base, suffix)) => {
            ensure!(suffix.ends_with(')'), "Invalid type size expression");
            (base.trim(), Some(&suffix[..suffix.len() - 1]))
        }
        None => (value.as_str(), None),
    };
    const TYPES: &[&str] = &[
        "INT", "INTEGER", "TINYINT", "SMALLINT", "MEDIUMINT", "BIGINT",
        "UNSIGNED BIG INT", "INT2", "INT8", "CHAR", "CHARACTER", "VARCHAR",
        "VARYING CHARACTER", "NCHAR", "NATIVE CHARACTER", "NATIONAL CHARACTER",
        "NVARCHAR", "NATIONAL VARYING CHARACTER", "TEXT", "CLOB", "BLOB",
        "REAL", "DOUBLE", "DOUBLE PRECISION", "FLOAT", "NUMERIC", "DECIMAL",
        "BOOLEAN", "DATE", "DATETIME", "ANY",
    ];
    ensure!(TYPES.contains(&base), "Unsupported SQLite declared type: {base}");

    if let Some(dimensions) = dimensions {
        ensure!(
            matches!(base, "CHAR" | "CHARACTER" | "VARCHAR" | "NCHAR" | "NATIONAL CHARACTER" | "NVARCHAR" | "NATIONAL VARYING CHARACTER" | "NUMERIC" | "DECIMAL"),
            "This declared type does not accept a size expression"
        );
        let parts = dimensions.split(',').map(str::trim).collect::<Vec<_>>();
        ensure!(
            (parts.len() == 1 || parts.len() == 2)
                && parts.iter().all(|part| part.parse::<u32>().is_ok_and(|number| number > 0)),
            "Type size must contain one or two positive integer dimensions"
        );
    } else {
        ensure!(!value.contains(')'), "Unexpected closing parenthesis in type name");
    }
    Ok(())
}

fn sql_literal(value: &Value) -> Result<String> {
    match value {
        Value::Null => Ok("NULL".to_owned()),
        Value::Integer(value) => Ok(value.to_string()),
        Value::Real(value) if value.is_finite() => Ok(value.to_string()),
        Value::Real(_) => bail!("A non-finite number cannot be used as a column default"),
        Value::Text(value) => Ok(format!("'{}'", value.replace('\'', "''"))),
        Value::Blob(bytes) => {
            let hex = bytes.iter().map(|byte| format!("{byte:02X}")).collect::<String>();
            Ok(format!("X'{hex}'"))
        }
    }
}

fn in_savepoint<T>(connection: &Connection, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    connection.execute_batch("SAVEPOINT oxide_studio_mutation")?;
    match operation() {
        Ok(value) => match connection.execute_batch("RELEASE SAVEPOINT oxide_studio_mutation") {
            Ok(()) => Ok(value),
            Err(error) => {
                rollback_savepoint(connection);
                Err(error.into())
            }
        },
        Err(error) => {
            rollback_savepoint(connection);
            Err(error)
        }
    }
}

fn rollback_savepoint(connection: &Connection) {
    if let Err(error) = connection.execute_batch(
        "ROLLBACK TO SAVEPOINT oxide_studio_mutation; RELEASE SAVEPOINT oxide_studio_mutation;",
    ) {
        tracing::error!("Could not roll back failed SQLite mutation savepoint: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn insert_update_and_delete_are_parameterized_and_keyed() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE records(id INTEGER PRIMARY KEY, label TEXT NOT NULL);").unwrap();
        insert_row(
            &connection,
            "records",
            &[("label".to_owned(), Value::Text("a'); DROP TABLE records; --".to_owned()))],
        ).unwrap();
        let affected = update_row(
            &connection,
            "records",
            &RowIdentity::PrimaryKey(vec![("id".to_owned(), Value::Integer(1))]),
            &[("label".to_owned(), Value::Text("updated".to_owned()))],
        ).unwrap();
        assert_eq!(affected, 1);
        assert_eq!(connection.query_row("SELECT label FROM records WHERE id=1", [], |row| row.get::<_, String>(0)).unwrap(), "updated");
        assert_eq!(delete_row(&connection, "records", &RowIdentity::PrimaryKey(vec![("id".to_owned(), Value::Integer(1))])).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn rowid_identity_works_when_table_has_no_primary_key() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE loose(value TEXT); INSERT INTO loose VALUES ('x');").unwrap();
        assert_eq!(delete_row(&connection, "loose", &RowIdentity::RowId { alias: "_rowid_".into(), value: 1 }).unwrap(), 1);
    }

    #[test]
    fn schema_operations_quote_identifiers_and_reject_type_injection() {
        let connection = Connection::open_in_memory().unwrap();
        create_table(&connection, "odd table", &[
            ColumnSpec { name: "id".into(), data_type: "INTEGER".into(), not_null: true, primary_key: true },
            ColumnSpec { name: "extra".into(), data_type: "TEXT".into(), not_null: false, primary_key: false },
        ]).unwrap();
        assert!(add_column(&connection, "odd table", "x", "TEXT); DROP TABLE", false, None).is_err());
        rename_column(&connection, "odd table", "id", "record id").unwrap();
        assert!(drop_column(&connection, "odd table", "extra").is_ok());
    }

    #[test]
    fn composite_primary_keys_target_exactly_one_row() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE composite(a INTEGER, b TEXT, value TEXT, PRIMARY KEY(a, b)); INSERT INTO composite VALUES (1, 'x', 'first'), (1, 'y', 'second');").unwrap();
        let identity = RowIdentity::PrimaryKey(vec![
            ("a".into(), Value::Integer(1)),
            ("b".into(), Value::Text("y".into())),
        ]);
        assert_eq!(update_row(&connection, "composite", &identity, &[("value".into(), Value::Text("updated".into()))]).unwrap(), 1);
        assert_eq!(delete_row(&connection, "composite", &identity).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT value FROM composite", [], |row| row.get::<_, String>(0)).unwrap(), "first");
    }

    #[test]
    fn failed_mutation_savepoint_preserves_outer_transaction() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE unique_values(id INTEGER PRIMARY KEY, value TEXT UNIQUE); BEGIN IMMEDIATE; INSERT INTO unique_values VALUES (1, 'kept');").unwrap();
        let failed = insert_row(&connection, "unique_values", &[
            ("id".into(), Value::Integer(2)),
            ("value".into(), Value::Text("kept".into())),
        ]);
        assert!(failed.is_err());
        assert!(!connection.is_autocommit());
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM unique_values", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        connection.execute_batch("COMMIT").unwrap();
    }

    #[test]
    fn type_grammar_accepts_common_types_and_rejects_constraint_injection() {
        assert!(validate_type_name("VARCHAR(255)").is_ok());
        assert!(validate_type_name("DECIMAL(10, 2)").is_ok());
        assert!(validate_type_name("TEXT CHECK(1)").is_err());
        assert!(validate_type_name("INTEGER); DROP TABLE x; --").is_err());
    }
}
