//! Application bootstrap: initialize the database pool and runtime services.

use ferro::DB;

/// Initialize services needed by the running server.
pub async fn register() {
    DB::init().await.unwrap_or_else(|e| {
        eprintln!("Error: failed to connect to database: {e}");
        eprintln!("  1. Set DATABASE_URL in .env (e.g. sqlite://./database.db)");
        eprintln!("  2. Ensure the database is reachable");
        std::process::exit(1);
    });
}
