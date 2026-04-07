# Static .gitignore — Phase 122.2 §8

# rust
/target
Cargo.lock

# node
frontend/node_modules
frontend/dist

# build
/public/assets

# generated_types
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
