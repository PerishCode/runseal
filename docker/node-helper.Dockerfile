FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    bash \
    ca-certificates \
    curl \
    git \
    jq \
    python3 \
    tar \
    xz-utils \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/runseal

CMD ["bash", "-lc", "sleep infinity"]
