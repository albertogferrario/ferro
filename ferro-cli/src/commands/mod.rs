pub mod api_check;
pub mod auth_link;
pub mod boost_install;
pub mod ci_init;
pub mod claude_install;
pub mod clean;
pub mod db_fresh;
pub mod db_migrate;
pub mod db_query;
pub mod db_rollback;
pub mod db_seed;
pub mod db_status;
pub mod db_sync;
pub mod deploy_init;
pub mod do_init;
pub mod docker_compose;
pub mod docker_init;
pub mod doctor;
pub mod generate_routes;
pub mod generate_types;
pub mod json_ui_schema;
pub mod make_action;
pub mod make_api;
pub mod make_api_key;
pub mod make_auth;
pub mod make_controller;
pub mod make_error;
pub mod make_event;
pub mod make_factory;
pub mod make_inertia;
pub mod make_job;
pub mod make_json_view;
pub mod make_lang;
pub mod make_listener;
pub mod make_middleware;
pub mod make_migration;
pub mod make_module;
pub mod make_notification;
pub mod make_policy;
pub mod make_projection;
pub mod make_resource;
pub mod make_scaffold;
pub mod make_seeder;
pub mod make_stripe;
pub mod make_task;
pub mod make_theme;
pub mod make_whatsapp;
pub mod mcp;
pub mod new;
#[cfg(feature = "projections")]
pub mod projection_check;
pub mod schedule_list;
pub mod schedule_run;
pub mod schedule_work;
pub mod serve;
pub mod storage_link;
pub mod validate_contracts;

/// Process-wide lock for tests that mutate `std::env::current_dir`.
///
/// `set_current_dir` is global state, so parallel tests that touch it race.
/// Tests should acquire this mutex before calling `set_current_dir` and hold
/// it until they restore the previous directory.
#[cfg(test)]
pub(crate) static CWD_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
