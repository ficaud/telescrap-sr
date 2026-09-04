# SPDX-License-Identifier: GPL-3.0-or-later

# Devcontainer for the telescrap-sr Rust workspace.
# Includes: Rust toolchain (rustup stable + clippy/rustfmt/rust-analyzer),
#           native build deps (openssl vendored + curl-sys), Neovim/LazyVim,
#           Oh My Zsh, Lazygit and OpenCode (best-effort developer tools).

FROM ubuntu:24.04

LABEL description="Rust dev environment for telescrap-sr"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# ------------------------------------------------------------------
# 1. System dependencies
# ------------------------------------------------------------------
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    perl \
    make \
    ca-certificates \
    curl \
    wget \
    git \
    unzip \
    xz-utils \
    zsh \
    sudo \
    ripgrep \
    fd-find \
    openssh-client \
    xclip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------
# 2. Rust toolchain (rustup stable + clippy/rustfmt/rust-analyzer)
# ------------------------------------------------------------------
# Installed system-wide under /usr/local so every user (and the runtime
# "tscrap" user created below) can use cargo/rustc/rust-analyzer.
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain stable \
        --component clippy,rustfmt,rust-analyzer,rust-src \
    && rustc --version \
    && cargo --version \
    && rust-analyzer --version

# Make the cargo binaries available to login shells and to `sudo` (sudo resets
# PATH to its secure_path, which would otherwise hide cargo/rustc/rustup/
# rust-analyzer in the entrypoint's privilege-drop step).
RUN echo 'export PATH="/usr/local/cargo/bin:$PATH"' > /etc/profile.d/cargo.sh \
    && echo 'Defaults secure_path="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/local/cargo/bin"' > /etc/sudoers.d/secure_path \
    && chmod 440 /etc/sudoers.d/secure_path

# ------------------------------------------------------------------
# 3. Neovim, Lazygit and OpenCode (optional developer tools)
# ------------------------------------------------------------------
# These downloads are developer conveniences, not required to build or
# test the project. A transient GitHub download failure must NOT abort
# the docker build, so each tool is installed in an isolated step that
# tolerates failure (logs a warning and continues).
ENV NVIM_VERSION=0.12.5
ENV LAZYGIT_VERSION=0.45.2

# Map the build architecture to the GitHub release asset suffix.
RUN ARCH="$(uname -m)" \
    && case "${ARCH}" in \
        x86_64) N_ARCH="x86_64" ;; \
        aarch64|arm64) N_ARCH="arm64" ;; \
        *) echo "Unsupported architecture: ${ARCH}" && exit 1 ;; \
    esac \
    && echo "${N_ARCH}" > /tmp/n_arch.txt

# 3a. Neovim (best-effort; failure is non-fatal)
RUN N_ARCH="$(cat /tmp/n_arch.txt)" \
    && NVIM_URL="https://github.com/neovim/neovim/releases/download/v${NVIM_VERSION}/nvim-linux-${N_ARCH}.tar.gz" \
    && echo "Downloading Neovim ${NVIM_VERSION} for ${N_ARCH}..." \
    && if wget -q --tries=3 --timeout=60 -O /tmp/nvim.tar.gz "${NVIM_URL}"; then \
         tar -xzf /tmp/nvim.tar.gz -C /opt/ \
         && ln -sf /opt/nvim-linux-${N_ARCH}/bin/nvim /usr/local/bin/nvim \
         && rm /tmp/nvim.tar.gz \
         && nvim --version | head -1 \
         && echo "Neovim installed" \
         ; \
       else \
         echo "WARNING: Neovim download failed; continuing without it" \
         ; \
       fi

# 3b. Lazygit (best-effort; failure is non-fatal)
RUN N_ARCH="$(cat /tmp/n_arch.txt)" \
    && LAZYGIT_URL="https://github.com/jesseduffield/lazygit/releases/download/v${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION}_Linux_${N_ARCH}.tar.gz" \
    && echo "Downloading Lazygit ${LAZYGIT_VERSION} for ${N_ARCH}..." \
    && if wget -q --tries=3 --timeout=60 -O /tmp/lazygit.tar.gz "${LAZYGIT_URL}"; then \
         tar -xzf /tmp/lazygit.tar.gz -C /usr/local/bin/ lazygit \
         && rm /tmp/lazygit.tar.gz \
         && lazygit --version \
         && echo "Lazygit installed" \
         ; \
       else \
         echo "WARNING: Lazygit download failed; continuing without it" \
         ; \
       fi \
    && rm -f /tmp/n_arch.txt

# 3b-bis. Lazygit configuration — enable overrideGpg to allow
# interactive commit editing when GPG signing is enabled.
RUN mkdir -p /home/${USERNAME}/.config/lazygit \
    && printf 'git:\n  overrideGpg: true\n' \
       > /home/${USERNAME}/.config/lazygit/config.yml \
    && chown -R ${USERNAME}:${USER_GID} /home/${USERNAME}/.config/lazygit

# 3c. OpenCode (best-effort; failure is non-fatal)
# AI coding agent for the terminal. The official installer only writes to
# $HOME/.opencode/bin, so we run it inside a temporary home and copy the
# binary to /usr/local/bin to put it on PATH for every user.
RUN OPENCODE_HOME="$(mktemp -d)" \
    && echo "Installing OpenCode..." \
    && if HOME="${OPENCODE_HOME}" bash -c "curl -fsSL https://opencode.ai/install | bash"; then \
         install -m 0755 "${OPENCODE_HOME}/.opencode/bin/opencode" /usr/local/bin/opencode \
         && opencode --version \
         && echo "OpenCode installed"; \
       else \
         echo "WARNING: OpenCode install failed; continuing without it"; \
       fi \
    && rm -rf "${OPENCODE_HOME}"

# 3d. Tree-sitter CLI (required by nvim-treesitter to compile parsers)
# LazyVim's nvim-treesitter needs the `tree-sitter` CLI to compile language
# parsers (e.g. Rust). Without it, opening a .rs file fails with
# "No parser for language rust". Install the prebuilt binary; best-effort so
# a transient download failure does not abort the build.
ENV TREE_SITTER_VERSION=0.27.0

RUN ARCH="$(uname -m)" \
    && case "${ARCH}" in \
        x86_64) TS_ARCH="x64" ;; \
        aarch64|arm64) TS_ARCH="arm64" ;; \
        *) echo "Unsupported architecture: ${ARCH}" && exit 1 ;; \
    esac \
    && TS_URL="https://github.com/tree-sitter/tree-sitter/releases/download/v${TREE_SITTER_VERSION}/tree-sitter-cli-linux-${TS_ARCH}.zip" \
    && echo "Downloading tree-sitter CLI ${TREE_SITTER_VERSION} for ${TS_ARCH}..." \
    && if wget -q --tries=3 --timeout=60 -O /tmp/tree-sitter.zip "${TS_URL}"; then \
         unzip -o /tmp/tree-sitter.zip -d /usr/local/bin/ \
         && rm /tmp/tree-sitter.zip \
         && chmod +x /usr/local/bin/tree-sitter \
         && tree-sitter --version \
         && echo "tree-sitter CLI installed" \
         ; \
       else \
         echo "WARNING: tree-sitter CLI download failed; continuing without it" \
         ; \
       fi

# ------------------------------------------------------------------
# 4. Non-root user
# ------------------------------------------------------------------
ARG USERNAME=tscrap
ARG USER_UID=1000
ARG USER_GID=1000

# The base image (ubuntu:24.04) ships a "ubuntu" user/group with UID/GID 1000.
# Remove it first so our "tscrap" user uniquely owns UID/GID 1000. This avoids
# duplicate-UID confusion (docker exec resolving UID 1000 to "ubuntu" and
# setting HOME=/home/ubuntu instead of /home/tscrap).
RUN userdel -r ubuntu 2>/dev/null || true; \
    groupdel ubuntu 2>/dev/null || true

# Create group first, then user, then assign sudo.
RUN groupadd --gid "${USER_GID}" "${USERNAME}" 2>/dev/null || true; \
    useradd --uid "${USER_UID}" --gid "${USER_GID}" -m "${USERNAME}"; \
    echo "${USERNAME} ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/"${USERNAME}" && \
    chmod 0440 /etc/sudoers.d/"${USERNAME}" && \
    echo "%sudo ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/sudogroup-nopasswd && \
    chmod 0440 /etc/sudoers.d/sudogroup-nopasswd

# ------------------------------------------------------------------
# 5. LazyVim (Neovim configuration) — optional developer tooling
# ------------------------------------------------------------------
# Best-effort clone; a transient GitHub failure must not abort the build.
RUN mkdir -p /home/${USERNAME}/.config \
    && if git clone --depth 1 https://github.com/LazyVim/starter /home/${USERNAME}/.config/nvim; then \
         rm -rf /home/${USERNAME}/.config/nvim/.git \
         && echo "LazyVim installed"; \
       else \
         echo "WARNING: LazyVim clone failed; continuing without it"; \
       fi \
    && chown -R ${USERNAME}:${USER_GID} /home/${USERNAME}/.config

# ------------------------------------------------------------------
# 6. Oh My Zsh + plugins (used by the host .zshrc mounted from the MacBook)
# ------------------------------------------------------------------
# The mounted .zshrc points ZSH="$HOME/.oh-my-zsh" and loads the plugins
# git, zsh-autosuggestions and zsh-syntax-highlighting.  We install those
# here (into /home/${USERNAME}) so the container is self-contained; the
# entrypoint bridges them to the effective runtime user.
# Oh My Zsh itself is CRITICAL: without oh-my-zsh.sh the mounted .zshrc
# fails to source and zsh is unusable.  Clone it with retries and abort the
# build if it never succeeds.
RUN clone_ohmyzsh() { \
         for i in 1 2 3 4 5; do \
           rm -rf /tmp/oh-my-zsh; \
           if git clone --depth 1 https://github.com/ohmyzsh/ohmyzsh.git /tmp/oh-my-zsh; then \
             return 0; \
           fi; \
           echo "Oh My Zsh clone attempt ${i} failed; retrying in 5s..."; \
           sleep 5; \
         done; \
         return 1; \
       }; \
       if clone_ohmyzsh; then \
         rm -rf /home/${USERNAME}/.oh-my-zsh \
         && mv /tmp/oh-my-zsh /home/${USERNAME}/.oh-my-zsh \
         && rm -rf /home/${USERNAME}/.oh-my-zsh/.git \
         && echo "Oh My Zsh installed"; \
       else \
         echo "ERROR: Oh My Zsh clone failed after retries" >&2; \
         exit 1; \
       fi \
    && test -f /home/${USERNAME}/.oh-my-zsh/oh-my-zsh.sh \
    && mkdir -p /home/${USERNAME}/.oh-my-zsh/custom/plugins \
    && if git clone --depth 1 https://github.com/zsh-users/zsh-autosuggestions /home/${USERNAME}/.oh-my-zsh/custom/plugins/zsh-autosuggestions; then \
         echo "zsh-autosuggestions installed"; \
       else \
         echo "WARNING: zsh-autosuggestions clone failed; continuing without it"; \
       fi \
    && if git clone --depth 1 https://github.com/zsh-users/zsh-syntax-highlighting /home/${USERNAME}/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting; then \
         echo "zsh-syntax-highlighting installed"; \
       else \
         echo "WARNING: zsh-syntax-highlighting clone failed; continuing without it"; \
       fi \
    && chown -R ${USERNAME}:${USER_GID} /home/${USERNAME}/.oh-my-zsh

# ------------------------------------------------------------------
# 7. Make zsh the default shell for the container user
# ------------------------------------------------------------------
RUN chsh -s /usr/bin/zsh ${USERNAME}

# Entrypoint — adjusts UID/GID at runtime for volume-mounted workspaces.
# Must run as root so it can call usermod/groupmod/chown, then
# it drops privileges to USERNAME via sudo -E before exec.
# Path is relative to build context (workspace root), set in devcontainer.json
COPY .devcontainer/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Switch back to root for entrypoint; the script will drop privileges
USER root
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
WORKDIR /workspaces/telescrap-sr
