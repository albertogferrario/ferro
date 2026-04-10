// Project scaffolding templates (used by `ferro new`)

// Backend templates

pub fn cargo_toml(package_name: &str, description: &str, author: &str) -> String {
    let authors_line = if author.is_empty() {
        String::new()
    } else {
        format!("authors = [\"{author}\"]\n")
    };

    format!(
        include_str!("files/backend/Cargo.toml.tpl"),
        package_name = package_name,
        description = description,
        authors_line = authors_line
    )
}

pub fn main_rs(package_name: &str) -> String {
    include_str!("files/backend/main.rs.tpl").replace("{package_name}", package_name)
}

pub fn routes_rs() -> &'static str {
    include_str!("files/backend/routes.rs.tpl")
}

pub fn controllers_mod() -> &'static str {
    include_str!("files/backend/controllers/mod.rs.tpl")
}

pub fn home_controller() -> &'static str {
    include_str!("files/backend/controllers/home.rs.tpl")
}

// Middleware templates

pub fn middleware_mod() -> &'static str {
    include_str!("files/backend/middleware/mod.rs.tpl")
}

pub fn middleware_logging() -> &'static str {
    include_str!("files/backend/middleware/logging.rs.tpl")
}

/// Template for models/mod.rs
pub fn models_mod() -> &'static str {
    include_str!("files/backend/models/mod.rs.tpl")
}

// Actions templates

pub fn actions_mod() -> &'static str {
    include_str!("files/backend/actions/mod.rs.tpl")
}

pub fn example_action() -> &'static str {
    include_str!("files/backend/actions/example_action.rs.tpl")
}

// Config templates

pub fn config_mod() -> &'static str {
    include_str!("files/backend/config/mod.rs.tpl")
}

pub fn config_database() -> &'static str {
    include_str!("files/backend/config/database.rs.tpl")
}

pub fn config_mail() -> &'static str {
    include_str!("files/backend/config/mail.rs.tpl")
}

pub fn bootstrap() -> &'static str {
    include_str!("files/backend/bootstrap.rs.tpl")
}

// Migrations templates

pub fn migrations_mod() -> &'static str {
    include_str!("files/backend/migrations/mod.rs.tpl")
}

// Frontend templates

pub fn package_json(project_name: &str) -> String {
    include_str!("files/frontend/package.json.tpl").replace("{project_name}", project_name)
}

pub fn vite_config() -> &'static str {
    include_str!("files/frontend/vite.config.ts.tpl")
}

pub fn tsconfig() -> &'static str {
    include_str!("files/frontend/tsconfig.json.tpl")
}

pub fn index_html(project_title: &str) -> String {
    include_str!("files/frontend/index.html.tpl").replace("{project_title}", project_title)
}

pub fn main_tsx() -> &'static str {
    include_str!("files/frontend/src/main.tsx.tpl")
}

pub fn home_page() -> &'static str {
    include_str!("files/frontend/src/pages/Home.tsx.tpl")
}

pub fn inertia_props_types() -> &'static str {
    include_str!("files/frontend/src/types/inertia-props.ts.tpl")
}

// Frontend layout templates

pub fn app_layout() -> &'static str {
    include_str!("files/frontend/src/layouts/AppLayout.tsx.tpl")
}

pub fn auth_layout() -> &'static str {
    include_str!("files/frontend/src/layouts/AuthLayout.tsx.tpl")
}

pub fn layouts_index() -> &'static str {
    include_str!("files/frontend/src/layouts/index.ts.tpl")
}

pub fn globals_css() -> &'static str {
    include_str!("files/frontend/src/styles/globals.css.tpl")
}

// Auth frontend templates

pub fn login_page() -> &'static str {
    include_str!("files/frontend/src/pages/auth/Login.tsx.tpl")
}

pub fn register_page() -> &'static str {
    include_str!("files/frontend/src/pages/auth/Register.tsx.tpl")
}

pub fn forgot_password_page() -> &'static str {
    include_str!("files/frontend/src/pages/auth/ForgotPassword.tsx.tpl")
}

pub fn reset_password_page() -> &'static str {
    include_str!("files/frontend/src/pages/auth/ResetPassword.tsx.tpl")
}

pub fn dashboard_page() -> &'static str {
    include_str!("files/frontend/src/pages/Dashboard.tsx.tpl")
}

pub fn profile_page() -> &'static str {
    include_str!("files/frontend/src/pages/Profile.tsx.tpl")
}

pub fn settings_page() -> &'static str {
    include_str!("files/frontend/src/pages/Settings.tsx.tpl")
}

// Auth backend templates

pub fn auth_controller() -> &'static str {
    include_str!("files/backend/controllers/auth.rs.tpl")
}

pub fn dashboard_controller() -> &'static str {
    include_str!("files/backend/controllers/dashboard.rs.tpl")
}

pub fn profile_controller() -> &'static str {
    include_str!("files/backend/controllers/profile.rs.tpl")
}

pub fn settings_controller() -> &'static str {
    include_str!("files/backend/controllers/settings.rs.tpl")
}

pub fn authenticate_middleware() -> &'static str {
    include_str!("files/backend/middleware/authenticate.rs.tpl")
}

pub fn user_model() -> &'static str {
    include_str!("files/backend/models/user.rs.tpl")
}

pub fn password_reset_tokens_model() -> &'static str {
    include_str!("files/backend/models/password_reset_tokens.rs.tpl")
}

// Auth migration templates

pub fn create_users_migration() -> &'static str {
    include_str!("files/backend/migrations/create_users_table.rs.tpl")
}

pub fn create_sessions_migration() -> &'static str {
    include_str!("files/backend/migrations/create_sessions_table.rs.tpl")
}

pub fn create_password_reset_tokens_migration() -> &'static str {
    include_str!("files/backend/migrations/create_password_reset_tokens_table.rs.tpl")
}

// Root templates

pub fn gitignore() -> &'static str {
    include_str!("files/root/gitignore.tpl")
}

pub fn env(project_name: &str) -> String {
    include_str!("files/root/env.tpl").replace("{project_name}", project_name)
}

pub fn env_example() -> &'static str {
    include_str!("files/root/env.example.tpl")
}

pub fn readme(project_name: &str, project_title: &str, description: &str) -> String {
    include_str!("files/root/README.md.tpl")
        .replace("{project_name}", project_name)
        .replace("{project_title}", project_title)
        .replace("{description}", description)
}

// Schedule templates

/// Template for schedule.rs registration file
pub fn schedule_rs() -> &'static str {
    include_str!("files/backend/schedule.rs.tpl")
}

/// Template for tasks/mod.rs
pub fn tasks_mod() -> &'static str {
    include_str!("files/backend/tasks/mod.rs.tpl")
}
