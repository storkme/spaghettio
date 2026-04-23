FROM node:24

RUN apt-get update && apt-get install -y --no-install-recommends \
      git gh jq less procps sudo curl ca-certificates \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN npm install -g @mariozechner/pi-coding-agent

RUN echo "node ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/node

USER node

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable \
      --component clippy --component rustfmt
ENV PATH="/home/node/.cargo/bin:$PATH"
RUN rustup target add wasm32-unknown-unknown

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY --chown=node:node docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
COPY --chown=node:node scripts/agent-runner.sh /usr/local/bin/agent-runner.sh
COPY --chown=node:node scripts/agents/ /usr/local/share/agents/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh /usr/local/bin/agent-runner.sh

WORKDIR /home/node

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["agent-runner.sh"]
