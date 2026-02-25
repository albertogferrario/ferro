//! Event and listener introspection

use crate::tools::list_events::{EventInfo, ListenerInfo};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, ItemStruct};
use walkdir::WalkDir;

struct EventVisitor {
    events: Vec<String>,
}

impl EventVisitor {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn has_event_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("Event")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for EventVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_event_derive(&node.attrs) {
            self.events.push(node.ident.to_string());
        }
        syn::visit::visit_item_struct(self, node);
    }
}

struct ListenerVisitor {
    listeners: Vec<(String, bool)>, // (name, is_queued)
}

impl ListenerVisitor {
    fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    fn has_listener_impl(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("Listener")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_queued(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("ShouldQueue")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for ListenerVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_listener_impl(&node.attrs) {
            let is_queued = self.is_queued(&node.attrs);
            self.listeners.push((node.ident.to_string(), is_queued));
        }
        syn::visit::visit_item_struct(self, node);
    }
}

pub fn scan_events(project_root: &Path) -> Vec<EventInfo> {
    let events_dir = project_root.join("src/events");
    let listeners_dir = project_root.join("src/listeners");

    let mut events_map: HashMap<String, (String, Vec<ListenerInfo>)> = HashMap::new();

    // Scan events directory
    if events_dir.exists() {
        for entry in WalkDir::new(&events_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(syntax) = syn::parse_file(&content) {
                    let mut visitor = EventVisitor::new();
                    visitor.visit_file(&syntax);

                    let relative_path = entry
                        .path()
                        .strip_prefix(project_root)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .to_string();

                    for event_name in visitor.events {
                        events_map.insert(event_name.clone(), (relative_path.clone(), Vec::new()));
                    }
                }
            }
        }
    }

    // Scan listeners directory
    if listeners_dir.exists() {
        for entry in WalkDir::new(&listeners_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(syntax) = syn::parse_file(&content) {
                    let mut visitor = ListenerVisitor::new();
                    visitor.visit_file(&syntax);

                    for (listener_name, is_queued) in visitor.listeners {
                        // Try to find which event this listener handles
                        // This is a simple heuristic - look for impl Listener<EventName>
                        for (event_name, (_, listeners)) in &mut events_map {
                            if content.contains(&format!("Listener<{event_name}>")) {
                                listeners.push(ListenerInfo {
                                    name: listener_name.clone(),
                                    queued: is_queued,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    events_map
        .into_iter()
        .map(|(name, (path, listeners))| EventInfo {
            name,
            path,
            listeners,
        })
        .collect()
}
