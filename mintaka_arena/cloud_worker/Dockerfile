FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends python3 git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY rust-toolchain.toml .
RUN rustup toolchain install --profile minimal
COPY . .

ARG TARGET_CPU=znver5
ARG BASE_REF=""
ARG BASE_PATCH=""
ARG TARGET_REF=""
ARG TARGET_PATCH=""
ENV RUSTFLAGS="-C target-cpu=${TARGET_CPU}"

RUN python3 -B -m mintaka_arena.build --base-ref="$BASE_REF" --base-patch="$BASE_PATCH" \
    --target-ref="$TARGET_REF" --target-patch="$TARGET_PATCH"

RUN rm -rf mintaka_arena/cloud_worker

FROM scratch
COPY --from=builder /app/mintaka_arena /mintaka_arena
COPY --from=builder /app/artifacts/engines /artifacts/engines
