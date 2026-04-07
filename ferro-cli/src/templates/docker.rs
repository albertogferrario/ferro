// ============================================================================
// Docker Templates
// ============================================================================
//
// Dockerfile rendering rewritten in Phase 122.2 Plan 06. Only the static
// .dockerignore and docker-compose helpers remain here for the surviving
// `docker:compose` command and the `new` project scaffold.

const DOCKERIGNORE_TPL: &str = include_str!("files/docker/dockerignore.tpl");

/// Static `.dockerignore` body. Phase 122.2 Plan 08 owns the canonical content.
pub fn dockerignore_template() -> &'static str {
    DOCKERIGNORE_TPL
}

/// Generate docker-compose.yml for local development
pub fn docker_compose_template(
    project_name: &str,
    include_mailpit: bool,
    include_minio: bool,
) -> String {
    let mailpit_service = if include_mailpit {
        include_str!("files/docker/mailpit.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let minio_service = if include_minio {
        include_str!("files/docker/minio.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let additional_volumes = if include_minio {
        "\n  minio_data:".to_string()
    } else {
        String::new()
    };

    include_str!("files/docker/docker-compose.yml.tpl")
        .replace("{project_name}", project_name)
        .replace("{mailpit_service}", &mailpit_service)
        .replace("{minio_service}", &minio_service)
        .replace("{additional_volumes}", &additional_volumes)
}
