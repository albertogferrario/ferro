use ferro_projections::{DataType, FieldMeaning, ServiceDef};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};

pub async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE items (id INTEGER PRIMARY KEY, status TEXT NOT NULL, customer_id INTEGER)"
            .to_string(),
    ))
    .await
    .expect("create table");
    for (id, status, cust) in [(1, "open", 10), (2, "open", 11), (3, "closed", 10)] {
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            format!(
                "INSERT INTO items (id, status, customer_id) VALUES ({id}, '{status}', {cust})"
            ),
        ))
        .await
        .expect("insert");
    }
    db
}

/// ServiceDef whose name "item" -> table "items" via the dispatch heuristic.
/// Marked mcp_exposed(true) so tools/list + tools/call see it.
pub fn item_service() -> ServiceDef {
    ServiceDef::new("item")
        .mcp_exposed(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
}
