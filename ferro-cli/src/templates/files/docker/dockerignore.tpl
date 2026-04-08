# Static .dockerignore — Phase 122.2 §8

# rust
target/

# node
frontend/node_modules/
frontend/dist/

# build
public/assets/

# ide
.idea/
.vscode/
*.swp
*.swo
.DS_Store

# env
.env
.env.*
!.env.example

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

# logs_tmp
*.log
tmp/

# git
.git/
.gitignore

# docker
Dockerfile
docker-compose*.yml
.dockerignore

# docs
*.md
# README.md is whitelisted so cargo's `readme = "README.md"` resolves at build time (ferro 127, D-20/D-21).
!README.md
LICENSE

# tests
tests/
