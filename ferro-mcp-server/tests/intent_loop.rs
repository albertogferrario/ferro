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
        let raw_list = include_str!(
            "fixtures/intent_loop/transcripts/list-orders.json"
        );
        let raw_approve = include_str!(
            "fixtures/intent_loop/transcripts/approve-order.json"
        );
        let raw_cancel = include_str!(
            "fixtures/intent_loop/transcripts/cancel-order.json"
        );
        let raw_ambiguous = include_str!(
            "fixtures/intent_loop/transcripts/ambiguous.json"
        );

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
                .map(|t| t == fixture.recorded_selection["tool_name"].as_str().unwrap_or(""))
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
        let fixture: IntentTurnFixture =
            serde_json::from_str(raw).expect("must parse");

        let provider = ReplayClassificationProvider::from_fixtures(&[fixture]);
        let result = provider
            .classify_raw(
                "system",
                "unknown NL message not in any fixture",
                &serde_json::json!({}),
                &ClassifierConfig::default(),
            )
            .await;

        assert!(
            result.is_err(),
            "unknown message must return error"
        );
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
    #[tokio::test]
    #[ignore]
    async fn intent_loop_live_eval() {
        if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") {
            return;
        }
        // Announce cost BEFORE first API call (isolate-before-spend discipline).
        eprintln!(
            "FERRO_AI_LIVE_EVAL=1: running live classification \
             (~4 turns × ~$0.005/call ≈ $0.02)"
        );
        // Live path implemented in Plan 03 when AnthropicProvider is wired.
        // This stub ensures the #[ignore] gate compiles and is exercisable.
        todo!("live eval wired in Plan 03 when process_nl_turn is available");
    }
}
