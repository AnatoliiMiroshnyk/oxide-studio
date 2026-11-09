use std::time::Instant;

use anyhow::{bail, Result};
use rusqlite::Connection;

use super::value::display_value;

pub const MAX_QUERY_ROWS: usize = 1_000;

#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub rows_affected: Option<usize>,
    pub elapsed_ms: u128,
    pub truncated: bool,
}

pub(crate) fn execute(connection: &Connection, sql: &str) -> Result<QueryResult> {
    if sql.trim().is_empty() {
        bail!("The SQL editor is empty");
    }

    let started = Instant::now();
    let mut statement = connection.prepare(sql)?;
    let column_count = statement.column_count();

    let mut result = QueryResult {
        columns: statement.column_names().map(str::to_owned).collect(),
        ..Default::default()
    };

    if column_count > 0 {
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            if result.rows.len() == MAX_QUERY_ROWS {
                result.truncated = true;
                break;
            }

            let mut values = Vec::with_capacity(column_count);
            for index in 0..column_count {
                values.push(display_value(row.get_ref(index)?));
            }
            result.rows.push(values);
        }
    } else {
        result.rows_affected = Some(statement.execute([])?);
    }

    result.elapsed_ms = started.elapsed().as_millis();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_output_is_capped_and_marked_as_truncated() {
        let connection = Connection::open_in_memory().unwrap();
        let result = execute(
            &connection,
            "WITH RECURSIVE numbers(n) AS (VALUES(1) UNION ALL SELECT n + 1 FROM numbers WHERE n < 1001) SELECT n FROM numbers",
        ).unwrap();

        assert_eq!(result.rows.len(), MAX_QUERY_ROWS);
        assert!(result.truncated);
        assert_eq!(result.rows[0][0], "1");
        assert_eq!(result.rows[MAX_QUERY_ROWS - 1][0], "1000");
    }
}
