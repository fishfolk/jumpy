# syntax=docker/dockerfile:1.4


#
# Jumpy Matchmaker Docker image
#

FROM rust:1.64-slim as builder

RUN apt-get update && \
    apt-get install -y \
        curl \
        pkg-config \
        libudev-dev \
        libasound2-dev && \
        rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/jumpy
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/src/jumpy/target \
    cargo build -p jumpy-matchmaker

RUN --mount=type=cache,target=/usr/src/jumpy/target \
    cp target/debug/jumpy-matchmaker /usr/local/bin/jumpy-matchmaker

# TODO: Slim down this container. We need to try and strip all unneeded deps from Bevy for the
# matchmaker.
FROM debian:bullseye
RUN apt-get update && apt-get install -y libasound2 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/jumpy-matchmaker /usr/local/bin/jumpy-matchmaker
COPY --from=builder /usr/src/jumpy/assets /usr/local/share/jumpy/assets
ENV JUMPY_ASSET_DIR=/usr/local/share/jumpy/assets
EXPOSE 8943/udp
ENTRYPOINT /usr/local/bin/jumpy-matchmaker