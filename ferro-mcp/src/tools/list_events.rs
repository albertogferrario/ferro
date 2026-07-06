//! List events tool - scan for events and listeners

use crate::error::Result;
use crate::introspection::events;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct EventsInfo {
    pub events: Vec<EventInfo>,
}

#[derive(Debug, Serialize)]
pub struct EventInfo {
    pub name: String,
    pub path: String,
    pub listeners: Vec<ListenerInfo>,
}

#[derive(Debug, Serialize)]
pub struct ListenerInfo {
    pub name: String,
    pub queued: bool,
}

pub fn execute(project_root: &Path) -> Result<EventsInfo> {
    let events_info = events::scan_events(project_root);
    Ok(EventsInfo {
        events: events_info,
    })
}
