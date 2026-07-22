//! Minimal Dashboard — DX-01 pit-of-success proof.
//!
//! Demonstrates that a ferro app composing standard json-ui components
//! (DashboardLayout + PageHeader + DataTable + StatCard + Form) produces
//! Linear/Attio-quality output with zero custom CSS.
//!
//! Run: `cargo run -p minimal-dashboard`
//! Then open: http://localhost:8099
//!
//! No tokens.css override, no inline styles, no custom CSS files.
//! Default ferro-json-ui tokens only.

use ferro::{
    get, handler, routes, serde_json, Config, DashboardLayout, DashboardLayoutConfig, HeaderProps,
    JsonUi, Response, Server, SessionConfig, SessionMiddleware, SidebarProps, Theme,
    ThemeMiddleware,
};
use ferro::{global_middleware, register_layout};
use std::path::Path;

routes! {
    get!("/", dashboard_index),
    // Stub logout so the avatar menu Esci link does not 404.
    get!("/logout", logout_stub),
}

#[tokio::main]
async fn main() {
    // Load .env if present; ignore missing file.
    Config::init(Path::new("."));

    // Default port 8099 — avoids collision with gestiscilo (:8080) and preview (:8081).
    // Override via SERVER_PORT env var.
    if std::env::var("SERVER_PORT").is_err() {
        unsafe {
            std::env::set_var("SERVER_PORT", "8099");
        }
    }

    // Register the "dashboard" layout shell — sidebar + header persistent frame.
    // No sidebar items: the example has a single view.
    register_layout(
        "dashboard",
        DashboardLayout::new(DashboardLayoutConfig {
            sidebar: SidebarProps {
                fixed_top: vec![],
                groups: vec![],
                fixed_bottom: vec![],
            },
            header: HeaderProps {
                business_name: "Demo App".to_string(),
                notification_count: None,
                user_name: Some("Demo User".to_string()),
                user_avatar: None,
                logout_url: Some("/logout".to_string()),
                theme_url: None,
                profile_url: None,
            },
            sse_url: None,
        }),
    );

    // Session middleware (required by DashboardLayout dark-mode cookie read).
    global_middleware!(SessionMiddleware::new(SessionConfig::from_env()));

    // Inject default design tokens as a <style> block in every JSON-UI page head.
    // This is the only styling — no external CSS file, no tokens.css override.
    global_middleware!(ThemeMiddleware::new().default_theme(Theme::default_theme()));

    Server::from_config(register())
        .run()
        .await
        .expect("server error");
}

/// GET / — render the dashboard view with inline demo data.
///
/// All UI structure lives in views/dashboard.json.
/// Data is synthetic — no DB required.
#[handler]
pub async fn dashboard_index() -> Response {
    let data = serde_json::json!({
        // StatCard values
        "stat_clienti": "128",
        "stat_ordini": "47",
        "stat_ricavi": "€ 3.840,00",

        // DataTable rows — realistic sample names, statuses, dates
        "clienti": [
            {
                "id": 1,
                "nome": "Marco Rossi",
                "email": "marco.rossi@example.com",
                "stato": "Attivo",
                "data_iscrizione": "15 gen 2025"
            },
            {
                "id": 2,
                "nome": "Laura Bianchi",
                "email": "laura.bianchi@example.com",
                "stato": "Attivo",
                "data_iscrizione": "22 feb 2025"
            },
            {
                "id": 3,
                "nome": "Giovanni Ferrari",
                "email": "g.ferrari@example.com",
                "stato": "Inattivo",
                "data_iscrizione": "08 mar 2025"
            },
            {
                "id": 4,
                "nome": "Alessia Conti",
                "email": "alessia.conti@example.com",
                "stato": "Attivo",
                "data_iscrizione": "01 apr 2025"
            },
            {
                "id": 5,
                "nome": "Stefano Mancini",
                "email": "s.mancini@example.com",
                "stato": "In revisione",
                "data_iscrizione": "18 mag 2025"
            }
        ]
    });

    JsonUi::render_file("views/dashboard.json", data)
}

/// GET /logout — stub so the avatar menu Esci link does not 404.
#[handler]
pub async fn logout_stub() -> Response {
    ferro::redirect!("/").into()
}
