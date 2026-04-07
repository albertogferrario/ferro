mod ai_boost;
mod auth;
pub mod ci_workflow;
#[path = "do.rs"]
pub mod do_;
pub mod docker;
mod entity;
mod make;
pub mod module;
mod project;
mod scaffold;

pub use ai_boost::*;
pub use auth::*;
pub use docker::*;
pub use entity::*;
pub use make::*;
pub use project::*;
pub use scaffold::*;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Backend Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cargo_toml_substitution() {
        let result = cargo_toml("my_app", "A test app", "Test Author <test@example.com>");
        assert!(result.contains("name = \"my_app\""));
        assert!(result.contains("description = \"A test app\""));
        assert!(result.contains("authors = [\"Test Author <test@example.com>\"]"));
    }

    #[test]
    fn test_cargo_toml_empty_author() {
        let result = cargo_toml("my_app", "A test app", "");
        assert!(result.contains("name = \"my_app\""));
        assert!(!result.contains("authors = "));
    }

    #[test]
    fn test_main_rs_substitution() {
        let result = main_rs("my_app");
        assert!(result.contains("my_app"));
    }

    #[test]
    fn test_routes_rs_not_empty() {
        assert!(!routes_rs().is_empty());
        assert!(routes_rs().contains("routes"));
    }

    #[test]
    fn test_controllers_mod_not_empty() {
        assert!(!controllers_mod().is_empty());
        assert!(controllers_mod().contains("auth"));
        assert!(controllers_mod().contains("dashboard"));
        assert!(controllers_mod().contains("profile"));
        assert!(controllers_mod().contains("settings"));
    }

    #[test]
    fn test_home_controller_not_empty() {
        assert!(!home_controller().is_empty());
        assert!(home_controller().contains("async fn index"));
    }

    #[test]
    fn test_auth_controller_not_empty() {
        assert!(!auth_controller().is_empty());
        assert!(auth_controller().contains("login"));
        assert!(auth_controller().contains("register"));
    }

    #[test]
    fn test_dashboard_controller_not_empty() {
        assert!(!dashboard_controller().is_empty());
        assert!(dashboard_controller().contains("Dashboard"));
    }

    #[test]
    fn test_profile_controller_not_empty() {
        let content = profile_controller();
        assert!(!content.is_empty());
        assert!(content.contains("Profile"));
        assert!(content.contains("async fn"));
    }

    #[test]
    fn test_settings_controller_not_empty() {
        let content = settings_controller();
        assert!(!content.is_empty());
        assert!(content.contains("Settings"));
        assert!(content.contains("async fn"));
    }

    // -------------------------------------------------------------------------
    // Middleware Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_middleware_mod_not_empty() {
        assert!(!middleware_mod().is_empty());
        assert!(middleware_mod().contains("logging"));
    }

    #[test]
    fn test_middleware_template_substitution() {
        let result = middleware_template("auth", "AuthMiddleware");
        assert!(result.contains("auth middleware"));
        assert!(result.contains("pub struct AuthMiddleware"));
        assert!(result.contains("impl Middleware for AuthMiddleware"));
    }

    #[test]
    fn test_authenticate_middleware_not_empty() {
        assert!(!authenticate_middleware().is_empty());
        assert!(authenticate_middleware().contains("Middleware"));
    }

    // -------------------------------------------------------------------------
    // Model Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_models_mod_not_empty() {
        let content = models_mod();
        assert!(!content.is_empty());
        assert!(content.contains("user"));
        assert!(content.contains("password_reset_tokens"));
    }

    #[test]
    fn test_user_model_not_empty() {
        assert!(!user_model().is_empty());
        assert!(user_model().contains("User"));
    }

    #[test]
    fn test_password_reset_tokens_model_not_empty() {
        let content = password_reset_tokens_model();
        assert!(!content.is_empty());
        assert!(content.contains("password_reset_tokens"));
        assert!(content.contains("email"));
        assert!(content.contains("token"));
    }

    // -------------------------------------------------------------------------
    // Migration Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_migrations_mod_not_empty() {
        let content = migrations_mod();
        assert!(!content.is_empty());
        assert!(content.contains("create_users_table"));
        assert!(content.contains("create_sessions_table"));
        assert!(content.contains("create_password_reset_tokens_table"));
    }

    #[test]
    fn test_create_users_migration_not_empty() {
        assert!(!create_users_migration().is_empty());
        assert!(create_users_migration().contains("Users"));
    }

    #[test]
    fn test_create_sessions_migration_not_empty() {
        assert!(!create_sessions_migration().is_empty());
        assert!(create_sessions_migration().contains("sessions"));
    }

    #[test]
    fn test_create_password_reset_tokens_migration_not_empty() {
        let content = create_password_reset_tokens_migration();
        assert!(!content.is_empty());
        assert!(content.contains("password_reset_tokens"));
    }

    // -------------------------------------------------------------------------
    // Config Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_config_mod_not_empty() {
        assert!(!config_mod().is_empty());
        assert!(config_mod().contains("database"));
    }

    #[test]
    fn test_config_database_not_empty() {
        assert!(!config_database().is_empty());
    }

    #[test]
    fn test_config_mail_not_empty() {
        assert!(!config_mail().is_empty());
    }

    // -------------------------------------------------------------------------
    // Frontend Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_package_json_substitution() {
        let result = package_json("my-project");
        assert!(result.contains("\"name\": \"my-project-frontend\""));
    }

    #[test]
    fn test_vite_config_not_empty() {
        assert!(!vite_config().is_empty());
        assert!(vite_config().contains("vite"));
    }

    #[test]
    fn test_tsconfig_not_empty() {
        assert!(!tsconfig().is_empty());
        assert!(tsconfig().contains("compilerOptions"));
    }

    #[test]
    fn test_index_html_substitution() {
        let result = index_html("My App");
        assert!(result.contains("<title>My App</title>"));
    }

    #[test]
    fn test_main_tsx_not_empty() {
        assert!(!main_tsx().is_empty());
        assert!(main_tsx().contains("createInertiaApp"));
    }

    #[test]
    fn test_home_page_not_empty() {
        assert!(!home_page().is_empty());
        assert!(home_page().contains("Home"));
    }

    #[test]
    fn test_inertia_props_types_not_empty() {
        let content = inertia_props_types();
        assert!(!content.is_empty());
        assert!(content.contains("User"));
        assert!(content.contains("DashboardProps"));
        assert!(content.contains("ProfileProps"));
        assert!(content.contains("SettingsProps"));
    }

    // -------------------------------------------------------------------------
    // Frontend Layout Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_layout_not_empty() {
        let content = app_layout();
        assert!(!content.is_empty());
        assert!(content.contains("AppLayout"));
        assert!(content.contains("Sidebar"));
    }

    #[test]
    fn test_auth_layout_not_empty() {
        let content = auth_layout();
        assert!(!content.is_empty());
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_layouts_index_not_empty() {
        let content = layouts_index();
        assert!(!content.is_empty());
        assert!(content.contains("AppLayout"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_globals_css_not_empty() {
        let content = globals_css();
        assert!(!content.is_empty());
        // Tailwind CSS v4 uses @import "tailwindcss" instead of @tailwind directives
        assert!(content.contains("tailwindcss"));
    }

    // -------------------------------------------------------------------------
    // Frontend Auth Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_login_page_not_empty() {
        let content = login_page();
        assert!(!content.is_empty());
        assert!(content.contains("Login"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_register_page_not_empty() {
        let content = register_page();
        assert!(!content.is_empty());
        assert!(content.contains("Register"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_forgot_password_page_not_empty() {
        let content = forgot_password_page();
        assert!(!content.is_empty());
        assert!(content.contains("ForgotPassword"));
        assert!(content.contains("AuthLayout"));
    }

    #[test]
    fn test_reset_password_page_not_empty() {
        let content = reset_password_page();
        assert!(!content.is_empty());
        assert!(content.contains("ResetPassword"));
        assert!(content.contains("AuthLayout"));
    }

    // -------------------------------------------------------------------------
    // Frontend User Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dashboard_page_not_empty() {
        let content = dashboard_page();
        assert!(!content.is_empty());
        assert!(content.contains("Dashboard"));
        assert!(content.contains("AppLayout"));
    }

    #[test]
    fn test_profile_page_not_empty() {
        let content = profile_page();
        assert!(!content.is_empty());
        assert!(content.contains("Profile"));
        assert!(content.contains("AppLayout"));
    }

    #[test]
    fn test_settings_page_not_empty() {
        let content = settings_page();
        assert!(!content.is_empty());
        assert!(content.contains("Settings"));
        assert!(content.contains("AppLayout"));
    }

    // -------------------------------------------------------------------------
    // Controller Template Generation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_controller_template_substitution() {
        let result = controller_template("users");
        assert!(result.contains("users controller"));
        assert!(result.contains("#[handler]"));
    }

    // -------------------------------------------------------------------------
    // Action Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_action_template_substitution() {
        let result = action_template("create_user", "CreateUser");
        assert!(result.contains("create_user action"));
        assert!(result.contains("pub struct CreateUser"));
        assert!(result.contains("#[injectable]"));
    }

    #[test]
    fn test_actions_mod_not_empty() {
        assert!(!actions_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Error Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_template_substitution() {
        let result = error_template("UserNotFound");
        assert!(result.contains("UserNotFound error"));
        assert!(result.contains("pub struct UserNotFound"));
        assert!(result.contains("#[domain_error"));
    }

    // -------------------------------------------------------------------------
    // Inertia Page Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_inertia_page_template_substitution() {
        let result = inertia_page_template("Users");
        assert!(result.contains("export default function Users()"));
        assert!(result.contains("<h1"));
    }

    // -------------------------------------------------------------------------
    // Event/Listener/Job Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_event_template_substitution() {
        let result = event_template("user_registered", "UserRegistered");
        assert!(result.contains("UserRegistered"));
        assert!(result.contains("impl Event for UserRegistered"));
    }

    #[test]
    fn test_listener_template_substitution() {
        let result = listener_template("send_welcome_email", "SendWelcomeEmail", "UserRegistered");
        assert!(result.contains("SendWelcomeEmail"));
        assert!(result.contains("impl Listener<UserRegistered>"));
    }

    #[test]
    fn test_job_template_substitution() {
        let result = job_template("send_email", "SendEmail");
        assert!(result.contains("SendEmail"));
        assert!(result.contains("impl Job for SendEmail"));
    }

    #[test]
    fn test_events_mod_not_empty() {
        assert!(!events_mod().is_empty());
    }

    #[test]
    fn test_listeners_mod_not_empty() {
        assert!(!listeners_mod().is_empty());
    }

    #[test]
    fn test_jobs_mod_not_empty() {
        assert!(!jobs_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Notification Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_notification_template_substitution() {
        let result = notification_template("order_shipped", "OrderShipped");
        assert!(result.contains("OrderShipped"));
        assert!(result.contains("impl Notification for OrderShipped"));
    }

    #[test]
    fn test_notifications_mod_not_empty() {
        assert!(!notifications_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Task Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_task_template_substitution() {
        let result = task_template("cleanup_old_sessions", "CleanupOldSessions");
        assert!(result.contains("CleanupOldSessions"));
        assert!(result.contains("impl Task for CleanupOldSessions"));
    }

    #[test]
    fn test_tasks_mod_not_empty() {
        assert!(!tasks_mod().is_empty());
    }

    #[test]
    fn test_schedule_rs_not_empty() {
        assert!(!schedule_rs().is_empty());
    }

    // -------------------------------------------------------------------------
    // Seeder Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_seeder_template_substitution() {
        let result = seeder_template("users_seeder", "UsersSeeder");
        assert!(result.contains("UsersSeeder"));
        assert!(result.contains("impl Seeder for UsersSeeder"));
    }

    #[test]
    fn test_seeders_mod_not_empty() {
        assert!(!seeders_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Factory Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_factory_template_substitution() {
        let result = factory_template("user_factory", "UserFactory", "User");
        assert!(result.contains("UserFactory"));
        assert!(result.contains("impl Factory for UserFactory"));
    }

    #[test]
    fn test_factories_mod_not_empty() {
        assert!(!factories_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Policy Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_policy_template_substitution() {
        let result = policy_template("post_policy", "PostPolicy", "Post");
        assert!(result.contains("PostPolicy"));
        assert!(result.contains("impl Policy<Post>"));
    }

    #[test]
    fn test_policies_mod_not_empty() {
        assert!(!policies_mod().is_empty());
    }

    // -------------------------------------------------------------------------
    // Docker Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dockerignore_template_not_empty() {
        assert!(!dockerignore_template().is_empty());
    }

    #[test]
    fn test_docker_compose_template_basic() {
        let result = docker_compose_template("my_project", false, false);
        assert!(result.contains("my_project"));
        assert!(result.contains("postgres"));
    }

    #[test]
    fn test_docker_compose_template_with_mailpit() {
        let result = docker_compose_template("my_project", true, false);
        assert!(result.contains("mailpit"));
    }

    #[test]
    fn test_docker_compose_template_with_minio() {
        let result = docker_compose_template("my_project", false, true);
        assert!(result.contains("minio"));
    }

    // -------------------------------------------------------------------------
    // Root File Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gitignore_not_empty() {
        assert!(!gitignore().is_empty());
        assert!(gitignore().contains("target"));
    }

    #[test]
    fn test_env_substitution() {
        let result = env("my_project");
        assert!(result.contains("my_project"));
    }

    #[test]
    fn test_env_example_not_empty() {
        assert!(!env_example().is_empty());
    }

    // -------------------------------------------------------------------------
    // AI Development Boost Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_ferro_guidelines_template_not_empty() {
        let content = ferro_guidelines_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro Framework"));
    }

    #[test]
    fn test_cursor_rules_template_not_empty() {
        let content = cursor_rules_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    #[test]
    fn test_claude_md_template_not_empty() {
        let content = claude_md_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    #[test]
    fn test_copilot_instructions_template_not_empty() {
        let content = copilot_instructions_template();
        assert!(!content.is_empty());
        assert!(content.contains("Ferro"));
    }

    // -------------------------------------------------------------------------
    // Entity Generation Helper Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_template_generates_valid_rust() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
            ColumnInfo {
                name: "email".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: true,
                is_primary_key: false,
            },
        ];

        let result = entity_template("users", &columns);
        assert!(result.contains("table_name = \"users\""));
        assert!(result.contains("pub id: i32"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("pub email: Option<String>"));
        assert!(result.contains("#[sea_orm(primary_key)]"));
    }

    #[test]
    fn test_entity_template_handles_reserved_keywords() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "type".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
        ];

        let result = entity_template("items", &columns);
        assert!(result.contains("pub r#type: String"));
        assert!(result.contains("column_name = \"type\""));
    }

    #[test]
    fn test_user_model_template_generates_minimal_file() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                is_nullable: false,
                is_primary_key: true,
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "VARCHAR".to_string(),
                is_nullable: false,
                is_primary_key: false,
            },
        ];

        let result = user_model_template("users", "User", &columns);
        // Type alias for convenient access
        assert!(result.contains("pub type User = Model"));
        // Re-exports entity module
        assert!(result.contains("pub use super::entities::users::*"));
        // Users table should have Authenticatable impl
        assert!(result.contains("impl ferro::auth::Authenticatable for Model"));
        // Should NOT contain manual method implementations (now generated by FerroModel macro)
        assert!(!result.contains("pub fn query()"));
        assert!(!result.contains("pub fn create()"));
        assert!(!result.contains("pub struct UserBuilder"));
    }

    #[test]
    fn test_entity_template_includes_ferro_model_derive() {
        let columns = vec![ColumnInfo {
            name: "id".to_string(),
            col_type: "INTEGER".to_string(),
            is_nullable: false,
            is_primary_key: true,
        }];

        let result = entity_template("users", &columns);
        // Should include FerroModel in derives
        assert!(result.contains("FerroModel"));
        assert!(result.contains("use ferro::FerroModel"));
    }

    #[test]
    fn test_entities_mod_template() {
        let tables = vec![
            TableInfo {
                name: "users".to_string(),
                columns: vec![],
            },
            TableInfo {
                name: "posts".to_string(),
                columns: vec![],
            },
        ];

        let result = entities_mod_template(&tables);
        assert!(result.contains("pub mod users;"));
        assert!(result.contains("pub mod posts;"));
    }

    // -------------------------------------------------------------------------
    // SQL Type Conversion Tests (via entity_template)
    // -------------------------------------------------------------------------

    #[test]
    fn test_sql_type_conversions() {
        let test_cases = vec![
            ("BIGINT", "i64"),
            ("INT8", "i64"),
            ("SMALLINT", "i16"),
            ("INT2", "i16"),
            ("INTEGER", "i32"),
            ("INT", "i32"),
            ("TEXT", "String"),
            ("VARCHAR(255)", "String"),
            ("CHAR(10)", "String"),
            ("BOOLEAN", "bool"),
            ("BOOL", "bool"),
            ("REAL", "f32"),
            ("FLOAT4", "f32"),
            ("DOUBLE", "f64"),
            ("FLOAT8", "f64"),
            ("TIMESTAMP", "DateTimeUtc"),
            ("DATETIME", "DateTimeUtc"),
            ("DATE", "Date"),
            ("TIME", "Time"),
            ("UUID", "Uuid"),
            ("JSON", "Json"),
            ("JSONB", "Json"),
            ("BYTEA", "Vec<u8>"),
            ("BLOB", "Vec<u8>"),
            ("DECIMAL", "Decimal"),
            ("NUMERIC", "Decimal"),
        ];

        for (sql_type, expected_rust_type) in test_cases {
            let columns = vec![ColumnInfo {
                name: "test_col".to_string(),
                col_type: sql_type.to_string(),
                is_nullable: false,
                is_primary_key: false,
            }];

            let result = entity_template("test_table", &columns);
            assert!(
                result.contains(&format!("pub test_col: {expected_rust_type}")),
                "Failed for SQL type '{sql_type}': expected Rust type '{expected_rust_type}' not found in:\n{result}"
            );
        }
    }

    #[test]
    fn test_nullable_types() {
        let columns = vec![ColumnInfo {
            name: "optional_name".to_string(),
            col_type: "VARCHAR".to_string(),
            is_nullable: true,
            is_primary_key: false,
        }];

        let result = entity_template("test_table", &columns);
        assert!(result.contains("pub optional_name: Option<String>"));
    }

    // -------------------------------------------------------------------------
    // API Controller Template Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_api_controller_template_substitution() {
        let result = api_controller_template(
            "Post",
            "post",
            "posts",
            "    pub title: String,\n    pub body: String,",
            "        .set_title(form.title.clone())\n        .set_body(form.body.clone())\n",
            "        title: sea_orm::ActiveValue::Set(form.title.clone()),\n        body: sea_orm::ActiveValue::Set(form.body.clone()),",
        );
        assert!(result.contains("Post API controller"));
        assert!(result.contains("pub async fn index"));
        assert!(result.contains("pub async fn show"));
        assert!(result.contains("pub async fn store"));
        assert!(result.contains("pub async fn update"));
        assert!(result.contains("pub async fn destroy"));
        assert!(result.contains("json_response!"));
        assert!(!result.contains("Inertia"));
        // Verify builder pattern is used in update handler
        assert!(result.contains(".update()"));
        assert!(result.contains(".set_title("));
        assert!(result.contains(".save()"));
        // Verify old ActiveModel pattern is NOT used in update handler
        assert!(!result.contains("let mut post: post::ActiveModel"));
    }
}
