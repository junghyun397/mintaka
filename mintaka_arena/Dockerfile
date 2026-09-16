FROM rust:alpine

RUN apk add --no-cache build-base git openssh-client python3

WORKDIR /app/mintaka

RUN git config --global --add safe.directory /app/mintaka

COPY . .

RUN rustup toolchain install --profile minimal

ENTRYPOINT ["python3", "-u", "mintaka_arena/arena_worker.py"]
