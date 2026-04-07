# Rust build artifacts
target/

# Node modules
frontend/node_modules/

# Frontend build output (built in Docker)
frontend/dist/

# IDE and editor files
.idea/
.vscode/
*.swp
*.swo
.DS_Store

# Environment files (provide at runtime)
.env
.env.*
!.env.example

# Git
.git/
.gitignore

# Docker files
Dockerfile
docker-compose*.yml
.dockerignore

# Build artifacts
public/assets/

# Logs and temp files
*.log
tmp/

# Documentation
*.md
LICENSE

# Test files
tests/

# Local databases and SQLite files
database.db
*.sqlite*

# Planning and workspace notes (never ship to Docker context)
.planning/

# User-uploaded files and runtime data
storage/
data/

# NOTE: .gitignore/.dockerignore drift audit deferred to Phase 124.
