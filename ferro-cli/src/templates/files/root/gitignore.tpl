# Static .gitignore — Phase 122.2 §8

# rust
/target
# Note: Cargo.lock is committed for binary crates to guarantee reproducible builds.

# node
frontend/node_modules
frontend/dist

# build
/public/assets

# generated_types — load-bearing: frontend/src/types/ is owned by `ferro generate-types`.
# Removing this rule breaks the generator-owned convention (see docs/src/cli/frontend-types.md).
frontend/src/types/

# ide
.idea
.vscode
*.swp
*.swo
.DS_Store

# env
.env
.env.local
.env.*.local
.env.production

# sqlite
database.db
*.sqlite*

# planning
.planning/

# storage
storage/
data/

# secrets
*.pem
*.key

# logs
*.log
tmp/
