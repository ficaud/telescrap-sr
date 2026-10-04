# Contributing to telescrap-sr

## Development environment (devcontainer)

The project ships a [devcontainer](.devcontainer/devcontainer.json).

### Prerequisites

- Docker Desktop (running)
- `devcontainer` CLI: `npm install -g @devcontainers/cli`

### Build and run

```bash
# Build the image only (without starting a container)
devcontainer build --workspace-folder .

# Build and start the container (reuses the existing one if already built)
devcontainer up --workspace-folder .

# Open a shell inside the running container (zsh is the default shell)
devcontainer exec --workspace-folder . zsh
```

### Rebuild

```bash
# Rebuild the image from scratch (no cache)
devcontainer build --workspace-folder . --no-cache

# Remove the existing container and start a fresh one
devcontainer up --workspace-folder . --remove-existing-container

# Full reset: rebuild without cache and recreate the container
devcontainer up --workspace-folder . --remove-existing-container --build-no-cache
```

### Stop and remove

The `devcontainer` CLI has no `stop`/`down` command; manage the container with
`docker`:

```bash
# Find the dev container (filtered by workspace folder)
docker ps -a --filter "label=devcontainer.local_folder=$(pwd)"

# Stop it
docker stop <container_name_or_id>

# Remove it
docker rm <container_name_or_id>

# Stop and remove in one step
docker rm -f <container_name_or_id>
```

