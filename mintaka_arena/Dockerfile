FROM rust:alpine

RUN apk add --no-cache build-base git openssh-client python3

WORKDIR /var/lib/mintaka

RUN git config --global --add safe.directory /var/lib/mintaka

COPY . .

RUN rustup toolchain install --profile minimal

ENTRYPOINT ["python3", "-u", "mintaka_arena/arena_worker.py"]
