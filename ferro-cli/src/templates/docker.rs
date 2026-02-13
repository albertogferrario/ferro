// ============================================================================
// Docker Templates
// ============================================================================

/// Generate Dockerfile for production deployment
pub fn dockerfile_template(package_name: &str) -> String {
    include_str!("files/docker/Dockerfile.tpl").replace("{package_name}", package_name)
}

/// Generate .dockerignore file
pub fn dockerignore_template() -> &'static str {
    include_str!("files/docker/dockerignore.tpl")
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

// ============================================================================
// DigitalOcean App Platform Templates
// ============================================================================

/// Generate app.yaml for DigitalOcean App Platform deployment
pub fn do_app_yaml_template(package_name: &str, github_repo: &str) -> String {
    include_str!("files/do/app.yaml.tpl")
        .replace("{package_name}", package_name)
        .replace("{github_repo}", github_repo)
}
