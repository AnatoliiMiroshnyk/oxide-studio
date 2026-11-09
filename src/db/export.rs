use std::path::Path;

use anyhow::Result;

use super::QueryResult;

pub(crate) fn write_csv(result: &QueryResult, path: impl AsRef<Path>) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().from_path(path)?;
    writer.write_record(&result.columns)?;
    for row in &result.rows {
        writer.write_record(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_headers_and_rows() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("output.csv");
        let result = QueryResult {
            columns: vec!["name".into(), "note".into()],
            rows: vec![vec!["Ada".into(), "a, b".into()]],
            ..Default::default()
        };
        write_csv(&result, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("name,note"));
        assert!(content.contains("\"a, b\""));
    }
}
