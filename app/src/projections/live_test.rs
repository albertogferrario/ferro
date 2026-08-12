use ferro::Event;
use ferro_projection::{Projection, ProjectionKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LiveTestEvent {
    pub increment: i32,
}

impl Event for LiveTestEvent {
    fn name(&self) -> &'static str {
        "LiveTestEvent"
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LiveTestState {
    pub count: i64,
}

#[derive(Clone, Serialize)]
pub struct LiveTestDelta {
    pub new_count: i64,
}

pub struct LiveTestProjection;

impl Projection for LiveTestProjection {
    type Event = LiveTestEvent;
    type State = LiveTestState;
    type Delta = LiveTestDelta;
    const NAME: &'static str = "live.test";

    fn key(&self, _event: &Self::Event) -> ProjectionKey {
        ProjectionKey::new("default")
    }

    fn apply(&self, state: &mut Self::State, event: &Self::Event) -> Self::Delta {
        state.count += event.increment as i64;
        LiveTestDelta {
            new_count: state.count,
        }
    }
}
