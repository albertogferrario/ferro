//! Application tests.

/// The `#[derive(Authenticatable)]` macro wires the trait from the `id` field,
/// removing the hand-written impl every user model used to need.
#[test]
fn user_authenticatable_derive() {
    use crate::models::user::User;
    use ferro::Authenticatable;

    let now = crate::models::now();
    let u = User {
        id: 7,
        name: "Alex".into(),
        email: "alex@nearly.app".into(),
        password: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    assert_eq!(u.auth_identifier(), 7);
    assert_eq!(u.auth_identifier_name(), "id");
    assert!(u.as_any().downcast_ref::<User>().is_some());
}

/// Each service projection must derive at least one intent — the core
/// projection / intent abstraction working end to end. The ServiceDefs remain
/// backend truth even though the UI is now Inertia/React.
#[test]
fn projections_derive_intents() {
    for svc in crate::projections::all() {
        let scores = ferro_projections::derive_intents(&svc);
        assert!(!scores.is_empty(), "a projection derived no intents");
    }
}

/// Presence expires: a position seen within the TTL is fresh, an old one is not.
#[test]
fn presence_freshness() {
    use crate::models::presence::{Presence, FRESH_TTL_MINUTES};

    let fresh = Presence {
        id: 1,
        user_id: 1,
        lat: 45.46,
        lng: 9.19,
        last_seen: crate::models::now(),
    };
    assert!(
        fresh.is_fresh(FRESH_TTL_MINUTES),
        "a just-seen presence is fresh"
    );

    let stale_ts =
        (chrono::Utc::now() - chrono::Duration::minutes(FRESH_TTL_MINUTES + 30)).to_rfc3339();
    let stale = Presence {
        id: 2,
        user_id: 2,
        lat: 45.46,
        lng: 9.19,
        last_seen: stale_ts,
    };
    assert!(
        !stale.is_fresh(FRESH_TTL_MINUTES),
        "a long-ago presence is stale"
    );
}

/// Product principle guard: Nearly has no messaging surface. The trillo carries
/// no message/body/text field (it *is* the whole payload), and the React pages
/// must not introduce a chat component or a message input.
#[test]
fn no_chat_surface() {
    // Backend: the trillo projection has no message-like field.
    let trillo = crate::projections::trillo::service_def();
    for f in &trillo.fields {
        assert!(
            !matches!(f.name.as_str(), "message" | "body" | "text" | "messaggio"),
            "trillo projection introduced a message field: {}",
            f.name
        );
    }

    // Frontend: no page ships a chat component or a message input.
    let pages = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/src/pages");
    let mut scanned = 0;
    if let Ok(entries) = std::fs::read_dir(pages) {
        for entry in entries.flatten() {
            scan_dir_for_chat(&entry.path(), &mut scanned);
        }
    }
    assert!(
        scanned > 0,
        "expected to scan React pages for a chat surface"
    );
}

#[cfg(test)]
fn scan_dir_for_chat(path: &std::path::Path, scanned: &mut usize) {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                scan_dir_for_chat(&entry.path(), scanned);
            }
        }
        return;
    }
    if path.extension().and_then(|e| e.to_str()) != Some("tsx") {
        return;
    }
    let src = std::fs::read_to_string(path).unwrap_or_default();
    *scanned += 1;
    for needle in [
        "<Chat",
        "name=\"message\"",
        "name=\"messaggio\"",
        "name=\"chat\"",
    ] {
        assert!(
            !src.contains(needle),
            "{}: introduces a messaging surface ({needle})",
            path.display()
        );
    }
}
