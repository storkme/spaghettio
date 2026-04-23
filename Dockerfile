FROM node:24

RUN apt-get update && apt-get install -y --no-install-recommends \
      git gh jq less procps sudo curl ca-certificates gettext-base \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

# Language servers the pi lsp-pi extension uses. typescript-language-server
# is installed globally via npm; rust-analyzer comes via rustup component
# (further down) after rust itself is installed.
RUN npm install -g @mariozechner/pi-coding-agent \
                   typescript \
                   typescript-language-server

RUN echo "node ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/node

USER node

# User-local npm prefix — `pi install npm:*` delegates to `npm install -g`
# which otherwise needs /usr/local write perms (root-only). This routes
# global installs into the node user's home instead.
RUN mkdir -p "$HOME/.npm-global" \
 && npm config set prefix "$HOME/.npm-global"
ENV PATH="/home/node/.npm-global/bin:/home/node/.cargo/bin:$PATH"

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable \
      --component clippy --component rustfmt --component rust-analyzer
RUN rustup target add wasm32-unknown-unknown

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# rtk (Rust Token Killer) — CLI proxy that trims common command output
# before it hits the model context. Compiled from source via cargo --git
# because the crates.io 'rtk' name is taken by an unrelated project.
RUN cargo install --git https://github.com/rtk-ai/rtk --locked

# pi extensions. These write to ~/.pi/settings.json and install npm packages
# into the user-local prefix we set above. `pi config` is an interactive TUI
# for toggling extensions — we don't invoke it; `pi install` is enough to
# register and enable.
RUN pi install npm:lsp-pi \
 && pi install npm:@sherif-fanous/pi-rtk \
 && pi install npm:pi-subagents \
 && pi install npm:@the-forge-flow/gh-pi

COPY --chown=node:node docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
COPY --chown=node:node scripts/agent-runner.sh /usr/local/bin/agent-runner.sh
COPY --chown=node:node scripts/agent-watcher.sh /usr/local/bin/agent-watcher.sh
COPY --chown=node:node scripts/agents/ /usr/local/share/agents/
COPY --chown=node:node scripts/pi/models.json.tmpl /usr/local/share/pi/models.json.tmpl
RUN chmod +x /usr/local/bin/docker-entrypoint.sh \
             /usr/local/bin/agent-runner.sh \
             /usr/local/bin/agent-watcher.sh

WORKDIR /home/node

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["agent-runner.sh"]
