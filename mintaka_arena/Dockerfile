FROM rust:bookworm

RUN apt-get update && apt-get install -y --no-install-recommends git openssh-client python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/mintaka

RUN git config --global --add safe.directory /app/mintaka

COPY . .

RUN rustup toolchain install --profile minimal

ENTRYPOINT ["bash", "/app/mintaka/mintaka_arena/docker.sh"]
