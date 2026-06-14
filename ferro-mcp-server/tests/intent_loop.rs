//! Replay test suite for the NL intent classification loop.
//!
//! Tests in this file are deterministic — they use `ReplayClassificationProvider`
//! to return pre-recorded `ToolSelection` fixtures without any network calls.
//!
//! The live eval path (real Anthropic API) is gated behind `#[ignore]` and the
//! `FERRO_AI_LIVE_EVAL=1` environment variable to prevent unintended spend.

#[cfg(feature = "ai")]
mod intent_loop {
    use async_trait::async_trait;
    use ferro_ai::{ClassificationProvider, ClassifierConfig, Error as AiError};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// A single recorded NL turn for replay.
    ///
    /// Mirrors the Phase 210 `Transcript`/`TrialRecord` pattern but simplified
    /// to a single turn (no multi-trial structure needed for classification replay).
    #[derive(Debug, Deserialize, Serialize)]
    pub struct IntentTurnFixture {
        pub turn_id: String,
        pub nl_message: String,
        /// The expected `tool_name` from the recorded_selection.
        pub expected_tool: String,
        /// Full `ToolSelection` JSON as returned by the classifier.
        pub recorded_selection: serde_json::Value,
    }

    /// Reqwest-free replay provider.
    ///
    /// Holds a map of NL message → recorded ToolSelection JSON and implements
    /// `ClassificationProvider` by returning the recorded value deterministically.
    /// Returns `Error::Provider` when no fixture matches the user_prompt.
    pub struct ReplayClassificationProvider {
        recordings: HashMap<String, serde_json::Value>,
    }

    impl ReplayClassificationProvider {
        pub fn from_fixtures(fixtures: &[IntentTurnFixture]) -> Self {
            let recordings = fixtures
                .iter()
                .map(|f| (f.nl_message.clone(), f.recorded_selection.clone()))
                .collect();
            Self { recordings }
        }
    }

    #[async_trait]
    impl ClassificationProvider for ReplayClassificationProvider {
        async fn classify_raw(
            &self,
            _system_prompt: &str,
            user_prompt: &str,
            _schema: &serde_json::Value,
            _config: &ClassifierConfig,
        ) -> Result<serde_json::Value, AiError> {
            self.recordings
                .get(user_prompt)
                .cloned()
                .ok_or_else(|| AiError::Provider {
                    status: None,
                    message: format!("no replay fixture for: {user_prompt}"),
                })
        }
    }

    /// Load all four fixtures, build the replay provider, and assert that
    /// `classify_raw` returns the recorded selection for each NL message.
    #[tokio::test]
    async fn fixtures_parse_and_replay_returns_recorded() {
        let raw_list = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let raw_approve = include_str!("fixtures/intent_loop/transcripts/approve-order.json");
        let raw_cancel = include_str!("fixtures/intent_loop/transcripts/cancel-order.json");
        let raw_ambiguous = include_str!("fixtures/intent_loop/transcripts/ambiguous.json");

        let f_list: IntentTurnFixture =
            serde_json::from_str(raw_list).expect("list-orders.json must parse");
        let f_approve: IntentTurnFixture =
            serde_json::from_str(raw_approve).expect("approve-order.json must parse");
        let f_cancel: IntentTurnFixture =
            serde_json::from_str(raw_cancel).expect("cancel-order.json must parse");
        let f_ambiguous: IntentTurnFixture =
            serde_json::from_str(raw_ambiguous).expect("ambiguous.json must parse");

        // Verify each fixture has a recorded_selection with expected fields.
        for fixture in [&f_list, &f_approve, &f_cancel, &f_ambiguous] {
            let sel = &fixture.recorded_selection;
            assert!(
                sel.get("tool_name").and_then(|v| v.as_str()).is_some(),
                "fixture {} must have tool_name string",
                fixture.turn_id
            );
            assert!(
                sel.get("confidence").and_then(|v| v.as_f64()).is_some(),
                "fixture {} must have confidence f64",
                fixture.turn_id
            );
            assert!(
                sel.get("arguments").is_some(),
                "fixture {} must have arguments object",
                fixture.turn_id
            );
        }

        let all_fixtures = [f_list, f_approve, f_cancel, f_ambiguous];
        let provider = ReplayClassificationProvider::from_fixtures(&all_fixtures);
        let schema = serde_json::json!({});
        let config = ClassifierConfig {
            confidence_threshold: 0.0,
            ..Default::default()
        };

        for fixture in &all_fixtures {
            let result = provider
                .classify_raw("system", &fixture.nl_message, &schema, &config)
                .await
                .expect("replay must return recorded selection");

            let returned_tool = result
                .get("tool_name")
                .and_then(|v| v.as_str())
                .expect("result must have tool_name");

            assert_eq!(
                returned_tool, fixture.expected_tool,
                "fixture {} replay returned wrong tool_name: {} (expected {})",
                fixture.turn_id, returned_tool, fixture.expected_tool
            );

            let returned_tool_matches_recorded = result
                .get("tool_name")
                .and_then(|v| v.as_str())
                .map(|t| {
                    t == fixture.recorded_selection["tool_name"]
                        .as_str()
                        .unwrap_or("")
                })
                .unwrap_or(false);
            assert!(
                returned_tool_matches_recorded,
                "fixture {} replay result must match recorded_selection.tool_name",
                fixture.turn_id
            );
        }
    }

    /// Miss path: unknown NL message returns Provider error.
    #[tokio::test]
    async fn replay_provider_returns_error_on_miss() {
        let raw = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");

        let provider = ReplayClassificationProvider::from_fixtures(&[fixture]);
        let result = provider
            .classify_raw(
                "system",
                "unknown NL message not in any fixture",
                &serde_json::json!({}),
                &ClassifierConfig::default(),
            )
            .await;

        assert!(result.is_err(), "unknown message must return error");
        match result.unwrap_err() {
            AiError::Provider { status, message } => {
                assert!(status.is_none(), "miss error status must be None");
                assert!(
                    message.contains("no replay fixture for"),
                    "miss error message must describe the miss; got: {message}"
                );
            }
            other => panic!("expected Provider error, got: {other:?}"),
        }
    }

    /// Live eval path — skipped in CI, opt-in via FERRO_AI_LIVE_EVAL=1.
    ///
    /// When enabled: makes real classification calls against the Anthropic API,
    /// asserts the returned tool_name matches the fixture's expected_tool, and
    /// announces the estimated cost before the first call.
    ///
    /// Run with:
    ///   FERRO_AI_LIVE_EVAL=1 ANTHROPIC_API_KEY=sk-ant-... \
    ///     cargo test -p ferro-mcp-server --features ai-live,confirmation \
    ///     -- --ignored intent_loop_live_eval
    ///
    /// Set FERRO_AI_UPDATE_FIXTURES=1 to overwrite committed fixtures on mismatch.
    #[cfg(feature = "ai-live")]
    #[tokio::test]
    #[ignore]
    async fn intent_loop_live_eval() {
        if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") {
            return;
        }

        // Load fixtures for the live eval run.
        let raw_list = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let raw_approve = include_str!("fixtures/intent_loop/transcripts/approve-order.json");
        let raw_cancel = include_str!("fixtures/intent_loop/transcripts/cancel-order.json");
        let raw_ambiguous = include_str!("fixtures/intent_loop/transcripts/ambiguous.json");

        let f_list: IntentTurnFixture =
            serde_json::from_str(raw_list).expect("list-orders.json must parse");
        let f_approve: IntentTurnFixture =
            serde_json::from_str(raw_approve).expect("approve-order.json must parse");
        let f_cancel: IntentTurnFixture =
            serde_json::from_str(raw_cancel).expect("cancel-order.json must parse");
        let f_ambiguous: IntentTurnFixture =
            serde_json::from_str(raw_ambiguous).expect("ambiguous.json must parse");

        let fixtures = [f_list, f_approve, f_cancel, f_ambiguous];

        // Cost announcement BEFORE the first API call (isolate-before-spend discipline, SC#4).
        eprintln!(
            "FERRO_AI_LIVE_EVAL=1: running live classification ({} turns x ~$0.005/call = ~${:.2})",
            fixtures.len(),
            fixtures.len() as f64 * 0.005
        );

        // Instantiate the live Anthropic provider (requires ANTHROPIC_API_KEY env var).
        let provider = std::sync::Arc::new(
            ferro_ai::AnthropicProvider::from_env()
                .expect("ANTHROPIC_API_KEY must be set for live eval"),
        );

        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "tool_name": { "type": "string", "description": "The tool to invoke" },
                "arguments": { "type": "object", "description": "Arguments for the tool" },
                "confidence": { "type": "number", "description": "Classifier confidence in [0.0, 1.0]" }
            },
            "required": ["tool_name", "arguments", "confidence"]
        });

        // Build a minimal system prompt for classification.
        let services = [test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let system = ferro_mcp_server::render_tool_descriptions(&services, &ctx)
            .expect("render_tool_descriptions must succeed");

        let classifier_config = ferro_ai::ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };

        let update_fixtures = std::env::var("FERRO_AI_UPDATE_FIXTURES").as_deref() == Ok("1");

        let mut mismatches: Vec<String> = Vec::new();

        for fixture in &fixtures {
            let classifier = ferro_ai::Classifier::<ferro_mcp_server::ToolSelection>::new(
                provider.clone(),
                classifier_config.clone(),
            );

            match classifier
                .classify(&system, &fixture.nl_message, &schema)
                .await
            {
                Ok(result) => {
                    let live_tool = &result.value.tool_name;
                    if live_tool != &fixture.expected_tool {
                        let msg = format!(
                            "fixture '{}': live returned '{}', expected '{}'",
                            fixture.turn_id, live_tool, fixture.expected_tool
                        );
                        if update_fixtures {
                            eprintln!("FERRO_AI_UPDATE_FIXTURES=1: would update fixture '{}' (manual step — rewrite the JSON file and recommit)", fixture.turn_id);
                        }
                        mismatches.push(msg);
                    } else {
                        eprintln!(
                            "fixture '{}': live classification MATCHED expected tool '{}'",
                            fixture.turn_id, live_tool
                        );
                    }
                }
                Err(ferro_ai::Error::LowConfidence {
                    best_guess,
                    confidence,
                }) => {
                    eprintln!(
                        "fixture '{}': low confidence ({:.0}%), best guess: {:?}",
                        fixture.turn_id,
                        confidence * 100.0,
                        best_guess.get("tool_name").and_then(|v| v.as_str())
                    );
                    // Low confidence on the ambiguous fixture is expected behaviour.
                    if fixture.turn_id != "ambiguous" {
                        mismatches.push(format!(
                            "fixture '{}': unexpected low confidence {:.2}",
                            fixture.turn_id, confidence
                        ));
                    }
                }
                Err(e) => {
                    mismatches.push(format!(
                        "fixture '{}': classification error: {}",
                        fixture.turn_id, e
                    ));
                }
            }
        }

        if !mismatches.is_empty() {
            panic!(
                "Live eval mismatches (set FERRO_AI_UPDATE_FIXTURES=1 to update fixtures):\n{}",
                mismatches.join("\n")
            );
        }
    }

    // ── End-to-end replay tests (SC#1 / SC#2 / SC#3 / SC#5) ─────────────────

    use ferro_mcp_server::WriteDispatcher;
    use ferro_projections::{ActionDef, DataType, FieldMeaning, InputDef, ServiceDef};
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// In-memory SQLite with the tables required by process_nl_turn's pipeline:
    /// - `orders` for the read (list_order) path
    /// - `mcp_idempotency_keys` for write idempotency
    /// - `audit_log` for write audit
    async fn setup_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                total REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tenant_id INTEGER NOT NULL
            )"
            .to_string(),
        ))
        .await
        .expect("create orders table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
                ('Alice', 100.0, 'pending', 1),
                ('Bob',   200.0, 'shipped', 1)"
                .to_string(),
        ))
        .await
        .expect("seed orders");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (tenant_id, idempotency_key)
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_idempotency_keys table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT,
                actor_kind TEXT NOT NULL,
                actor_id TEXT,
                action TEXT NOT NULL,
                target_kind TEXT,
                target_id TEXT,
                before TEXT,
                after TEXT,
                reason TEXT,
                correlation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create audit_log table");
        db
    }

    /// Build the test ServiceDef:
    /// - `list_order` (auto-derived read tool, mcp_exposed)
    /// - `approve` (non-destructive write — no transition_trigger)
    /// - `submit` (destructive write — transition_trigger.is_some())
    fn test_service() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            // approve: non-destructive (no transition_trigger)
            .action(ActionDef::new("approve").input(InputDef::new(
                "id",
                DataType::Integer,
                FieldMeaning::Identifier,
            )))
            // submit: destructive (has transition_trigger) — the D-08 seam fires
            .action(
                ActionDef::new("submit")
                    .transition_trigger("submit")
                    .input(InputDef::new(
                        "id",
                        DataType::Integer,
                        FieldMeaning::Identifier,
                    )),
            )
    }

    /// Build provider from a single fixture's recorded_selection, using the
    /// fixture's nl_message as the lookup key.
    fn single_fixture_provider(fixture: &IntentTurnFixture) -> Arc<ReplayClassificationProvider> {
        Arc::new(ReplayClassificationProvider::from_fixtures(
            std::slice::from_ref(fixture),
        ))
    }

    /// SC#3 / SC#1 (read branch): "show me the orders" → list_order → read path.
    ///
    /// Asserts: result envelope has content[] and isError:false; executor NOT called.
    #[tokio::test]
    async fn read_turn() {
        let raw = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");
        assert_eq!(fixture.expected_tool, "list_order");

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(serde_json::json!({ "status": "ok" })) })
                }
            }),
        };

        // Threshold 0.7 < confidence 0.95: classifier succeeds.
        let config = ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let provider = single_fixture_provider(&fixture);

        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| true,
            provider,
            config,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        // Must be a valid MCP envelope: has content[] and isError:false.
        let result_inner = &result["result"];
        assert!(
            result_inner.get("content").is_some(),
            "read result must have content[]; got: {result:?}"
        );
        assert_eq!(
            result_inner["isError"].as_bool(),
            Some(false),
            "read result must be isError:false; got: {result:?}"
        );
        // Executor must NOT have been called on the read path.
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT be invoked on the read path"
        );
    }

    /// SC#1 read-authorization regression (WR-01): a read turn whose ability gate
    /// DENIES must return an `access_denied` envelope and NOT dispatch. This proves
    /// the NL surface enforces the same app-ability gate as the direct `/mcp` path —
    /// a user denied a projection's `mcp_ability` cannot read it by phrasing the
    /// request in natural language.
    #[tokio::test]
    async fn read_denied_by_ability_gate() {
        let raw = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");
        assert_eq!(fixture.expected_tool, "list_order");

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(serde_json::json!({ "status": "ok" })) })
                }
            }),
        };

        let config = ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let provider = single_fixture_provider(&fixture);

        // Deny-all authorization closure: the read must be blocked before dispatch.
        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| false,
            provider,
            config,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        // No dispatch on a denied read.
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT be invoked when the read ability gate denies; got: {result:?}"
        );
        // Deny envelope: isError:true, structuredContent.status == access_denied.
        let result_inner = &result["result"];
        assert_eq!(
            result_inner["isError"].as_bool(),
            Some(true),
            "denied read must be isError:true; got: {result:?}"
        );
        assert_eq!(
            result_inner["structuredContent"]["status"].as_str(),
            Some("access_denied"),
            "denied read must return access_denied status; got: {result:?}"
        );
    }

    /// SC#3 / SC#1 (write branch): "approve the order from Alice" → approve →
    /// non-destructive write → executor invoked.
    ///
    /// Asserts: executor called once, result envelope isError:false.
    #[tokio::test]
    async fn write_turn() {
        let raw = include_str!("fixtures/intent_loop/transcripts/approve-order.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");
        assert_eq!(fixture.expected_tool, "approve");

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(serde_json::json!({ "status": "approved" })) })
                }
            }),
        };

        // Threshold 0.7 < confidence 0.92: classifier succeeds.
        let config = ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let provider = single_fixture_provider(&fixture);

        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| true,
            provider,
            config,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        // Executor must have been called exactly once.
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must be invoked for a non-destructive write; got: {result:?}"
        );
        let result_inner = &result["result"];
        assert_eq!(
            result_inner["isError"].as_bool(),
            Some(false),
            "write result must be isError:false; got: {result:?}"
        );
    }

    /// SC#3 / SC#2 (confirmation gate): "submit order 7" → submit → destructive
    /// write (transition_trigger.is_some()) → confirmation-required envelope,
    /// executor NOT invoked.
    #[cfg(feature = "confirmation")]
    #[tokio::test]
    async fn destructive_requires_confirm() {
        let raw = include_str!("fixtures/intent_loop/transcripts/cancel-order.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");
        assert_eq!(fixture.expected_tool, "submit");

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(serde_json::json!({ "status": "submitted" })) })
                }
            }),
        };

        // Threshold 0.7 < confidence 0.9: classifier succeeds.
        let config = ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let provider = single_fixture_provider(&fixture);

        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| true,
            provider,
            config,
            &dispatcher,
            &ferro_ai::InMemoryConfirmationStore::new(),
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        // Executor must NOT have been called — the D-08 seam blocks it.
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT be invoked for a destructive write without confirmation; got: {result:?}"
        );
        // The envelope must indicate confirmation is required.
        let result_inner = &result["result"];
        let error_kind = result_inner["structuredContent"]["error_kind"]
            .as_str()
            .unwrap_or("");
        assert_eq!(
            error_kind, "confirmation_required",
            "destructive write must return confirmation_required; got: {result:?}"
        );
    }

    /// SC#3 / SC#5 (low-confidence): "do the thing" (confidence 0.3 < threshold
    /// 0.7) → LowConfidence → needs_clarification envelope, no dispatch.
    #[tokio::test]
    async fn low_confidence() {
        let raw = include_str!("fixtures/intent_loop/transcripts/ambiguous.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");
        // Fixture has confidence 0.3, below default threshold 0.7.
        let recorded_confidence = fixture.recorded_selection["confidence"]
            .as_f64()
            .expect("must have confidence");
        assert!(
            recorded_confidence < 0.7,
            "ambiguous fixture must have confidence below 0.7; got {recorded_confidence}"
        );

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(serde_json::json!({ "status": "ok" })) })
                }
            }),
        };

        // Default threshold 0.7 > fixture confidence 0.3 → LowConfidence.
        let config = ClassifierConfig::default();
        let provider = single_fixture_provider(&fixture);

        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| true,
            provider,
            config,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        // Executor must NOT have been called.
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT be invoked on low-confidence; got: {result:?}"
        );
        // Result must be needs_clarification with isError:false.
        let result_inner = &result["result"];
        assert_eq!(
            result_inner["isError"].as_bool(),
            Some(false),
            "needs_clarification must have isError:false; got: {result:?}"
        );
        assert_eq!(
            result_inner["structuredContent"]["status"].as_str(),
            Some("needs_clarification"),
            "low-confidence must return needs_clarification status; got: {result:?}"
        );
        assert!(
            result_inner["structuredContent"]["question"]
                .as_str()
                .is_some(),
            "needs_clarification must include a question; got: {result:?}"
        );
        assert!(
            result_inner["structuredContent"]["best_guess"].is_object()
                || result_inner["structuredContent"]["best_guess"].is_null()
                || result_inner["structuredContent"]["best_guess"].is_string(),
            "needs_clarification must include best_guess; got: {result:?}"
        );
    }

    /// Phase 205 regression guard extended to turn outcomes (SC#3).
    ///
    /// Every result from `process_nl_turn` must have a `content` array and an
    /// `isError` bool inside the `result` key, matching the MCP CallToolResult shape.
    #[tokio::test]
    async fn turn_result_valid_mcp() {
        // Use the read turn as the reference case (simplest, no DB writes).
        let raw = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");

        let db = setup_db().await;
        let services = vec![test_service()];
        let ctx = ferro_mcp_server::McpContext::default();

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(serde_json::json!({ "status": "ok" })) })
            }),
        };

        let config = ClassifierConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let provider = single_fixture_provider(&fixture);

        let result = ferro_mcp_server::process_nl_turn(
            &fixture.nl_message,
            &services,
            &db,
            Some(1),
            &ctx,
            &|_| true,
            provider,
            config,
            &dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &ferro_mcp_server::McpServerConfig::default(),
        )
        .await;

        let result_inner = &result["result"];
        assert!(
            result_inner
                .get("content")
                .and_then(|v| v.as_array())
                .is_some(),
            "turn result must have content array; got: {result:?}"
        );
        assert!(
            result_inner
                .get("isError")
                .and_then(|v| v.as_bool())
                .is_some(),
            "turn result must have isError bool; got: {result:?}"
        );
    }

    /// Determinism assertion: the read and write turns return byte-identical
    /// structuredContent when run twice.
    #[tokio::test]
    async fn replay_deterministic() {
        let raw_list = include_str!("fixtures/intent_loop/transcripts/list-orders.json");
        let f_list: IntentTurnFixture = serde_json::from_str(raw_list).expect("must parse");
        let raw_approve = include_str!("fixtures/intent_loop/transcripts/approve-order.json");
        let f_approve: IntentTurnFixture = serde_json::from_str(raw_approve).expect("must parse");

        for fixture in [&f_list, &f_approve] {
            let db = setup_db().await;
            let services = vec![test_service()];
            let ctx = ferro_mcp_server::McpContext::default();

            let make_dispatcher = || WriteDispatcher {
                guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
                executor: Box::new(|_, _, _, _| {
                    Box::pin(async { Ok(serde_json::json!({ "status": "approved" })) })
                }),
            };

            let config = ClassifierConfig {
                confidence_threshold: 0.7,
                ..Default::default()
            };

            #[cfg(feature = "confirmation")]
            let store = ferro_ai::InMemoryConfirmationStore::new();
            #[cfg(feature = "confirmation")]
            let mcp_config = ferro_mcp_server::McpServerConfig::default();

            let provider1 = single_fixture_provider(fixture);
            let result1 = ferro_mcp_server::process_nl_turn(
                &fixture.nl_message,
                &services,
                &db,
                Some(1),
                &ctx,
                &|_| true,
                provider1,
                config.clone(),
                &make_dispatcher(),
                #[cfg(feature = "confirmation")]
                &store,
                #[cfg(feature = "confirmation")]
                &mcp_config,
            )
            .await;

            let provider2 = single_fixture_provider(fixture);
            let result2 = ferro_mcp_server::process_nl_turn(
                &fixture.nl_message,
                &services,
                &db,
                Some(1),
                &ctx,
                &|_| true,
                provider2,
                config.clone(),
                &make_dispatcher(),
                #[cfg(feature = "confirmation")]
                &store,
                #[cfg(feature = "confirmation")]
                &mcp_config,
            )
            .await;

            // Compare structuredContent (strip non-deterministic fields if any).
            let sc1 = &result1["result"]["structuredContent"];
            let sc2 = &result2["result"]["structuredContent"];
            assert_eq!(
                sc1, sc2,
                "fixture '{}': structuredContent must be identical on two runs",
                fixture.turn_id
            );
        }
    }
}
