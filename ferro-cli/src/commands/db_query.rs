//! db:query command - Execute raw SQL queries against the database

use console::style;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use std::env;

pub fn run(query: String) {
    // Load DATABASE_URL from .env
    dotenvy::dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!(
                "{} DATABASE_URL not set in .env",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    // Use tokio runtime for async database operations
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        execute_query(&database_url, &query).await;
    });
}

async fn execute_query(database_url: &str, query: &str) {
    let is_sqlite = database_url.starts_with("sqlite");
    let backend = if is_sqlite {
        DbBackend::Sqlite
    } else {
        DbBackend::Postgres
    };

    // Connect to database
    let db = match Database::connect(database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "{} Failed to connect to database: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
    };

    // Execute the query
    let result = db
        .query_all(Statement::from_string(backend, query.to_string()))
        .await;

    match result {
        Ok(rows) => {
            if rows.is_empty() {
                println!("{}", style("Empty result set").dim());
                return;
            }

            // Try to get column names from the first row
            // SeaORM doesn't expose column metadata directly, so we'll display raw values
            print_results(&rows);
        }
        Err(e) => {
            eprintln!("{} Query failed: {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn print_results(rows: &[sea_orm::QueryResult]) {
    if rows.is_empty() {
        return;
    }

    // For SELECT queries, we need to try to extract values
    // Since we don't know the schema, we'll try common column indices
    let mut table_data: Vec<Vec<String>> = Vec::new();

    for row in rows {
        let mut row_data: Vec<String> = Vec::new();

        // Try to extract values from indices 0-19 (reasonable column limit)
        for i in 0..20 {
            // Try different types in order of likelihood
            if let Ok(val) = row.try_get_by_index::<i32>(i) {
                row_data.push(val.to_string());
            } else if let Ok(val) = row.try_get_by_index::<i64>(i) {
                row_data.push(val.to_string());
            } else if let Ok(val) = row.try_get_by_index::<String>(i) {
                row_data.push(val);
            } else if let Ok(val) = row.try_get_by_index::<bool>(i) {
                row_data.push(val.to_string());
            } else if let Ok(val) = row.try_get_by_index::<f64>(i) {
                row_data.push(val.to_string());
            } else if let Ok(val) = row.try_get_by_index::<Option<String>>(i) {
                row_data.push(val.unwrap_or_else(|| "NULL".to_string()));
            } else if let Ok(val) = row.try_get_by_index::<Option<i32>>(i) {
                row_data.push(
                    val.map(|v| v.to_string())
                        .unwrap_or_else(|| "NULL".to_string()),
                );
            } else {
                // No more columns at this index
                break;
            }
        }

        if !row_data.is_empty() {
            table_data.push(row_data);
        }
    }

    if table_data.is_empty() {
        println!("{}", style("No data to display").dim());
        return;
    }

    // Calculate column widths
    let num_cols = table_data.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths: Vec<usize> = vec![0; num_cols];

    for row in &table_data {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len()).max(3);
            }
        }
    }

    // Print separator
    let separator: String = col_widths
        .iter()
        .map(|w| "-".repeat(*w + 2))
        .collect::<Vec<_>>()
        .join("+");
    println!("+{separator}+");

    // Print rows
    for row in &table_data {
        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = col_widths.get(i).copied().unwrap_or(10);
                format!(" {cell:width$} ")
            })
            .collect();
        println!("|{}|", formatted.join("|"));
    }

    println!("+{separator}+");
    println!("\n{} {} row(s)", style("→").cyan(), table_data.len());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_detect_sqlite_backend() {
        let url = "sqlite://test.db";
        assert!(url.starts_with("sqlite"));
    }

    #[test]
    fn test_detect_postgres_backend() {
        let url = "postgres://localhost/test";
        assert!(!url.starts_with("sqlite"));
    }

    #[test]
    fn test_column_width_calculation() {
        let table_data = vec![
            vec!["id".to_string(), "name".to_string()],
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
        ];

        let num_cols = table_data.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths: Vec<usize> = vec![0; num_cols];

        for row in &table_data {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len()).max(3);
                }
            }
        }

        assert_eq!(col_widths[0], 3); // "id" -> min 3
        assert_eq!(col_widths[1], 5); // "Alice" -> 5
    }

    #[test]
    fn test_separator_generation() {
        let col_widths = [3, 5, 10];
        let separator: String = col_widths
            .iter()
            .map(|w| "-".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("+");

        assert_eq!(separator, "-----+-------+------------");
    }

    #[test]
    fn test_row_formatting() {
        let row = ["1".to_string(), "Alice".to_string()];
        let col_widths = [3, 10];

        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = col_widths.get(i).copied().unwrap_or(10);
                format!(" {cell:width$} ")
            })
            .collect();

        assert_eq!(formatted[0], " 1   ");
        assert_eq!(formatted[1], " Alice      ");
    }
}
