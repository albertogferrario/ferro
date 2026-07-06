# ferro-deployments

Immutable deployment rows and atomic pointer promotion for the Ferro framework.

Each deployment is recorded as an immutable row. Promoting a deployment atomically
flips a per-owner pointer, preserving the previous deployment ID for rollback. The
artifact storage abstraction is backend-agnostic (local disk, S3, or any
`ferro-storage` driver).

## Usage

Register the migration helpers in your application's `Migrator`:

```rust,ignore
use ferro_deployments::{CreateDeploymentsTable, CreateDeploymentPointersTable};

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateDeploymentsTable),
            Box::new(CreateDeploymentPointersTable),
            // ... your application migrations
        ]
    }
}
```

## Configuration

`DeploymentConfig::from_env()` reads:

- `DEPLOYMENT_PREVIEW_DOMAIN` — optional domain for preview subdomain URLs.

## License

MIT
