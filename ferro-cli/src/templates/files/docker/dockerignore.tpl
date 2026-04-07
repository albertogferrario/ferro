# Generated from templates/files/root/ignore_patterns.toml — edit there, run ferro ignore:sync

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
LICENSE

# tests
tests/
