ARG BIN_MODE=indexer

FROM rust:1.95.0-slim-bookworm AS builder
WORKDIR /build
COPY . .
ARG BUILD_PROFILE=release
ARG BIN_MODE

RUN apt update && apt install -y git make libssl-dev pkg-config libpq-dev build-essential protobuf-compiler clang libclang-dev libgmp-dev libomp-dev libopenmpi-dev libsodium-dev \
    && cargo build --profile $BUILD_PROFILE --bin $BIN_MODE --locked \
    && cp /build/target/$BUILD_PROFILE/$BIN_MODE /build/vectorx-$BIN_MODE

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libpq5 libssl3 libsodium23 libgmp10 libgomp1 libopenmpi3 libstdc++6 adduser \
    && apt clean \
    && adduser --disabled-password --gecos "" --no-create-home --uid 1001 bridge

FROM runtime AS indexer

FROM node:22.18.0-bookworm-slim AS snarkjs
RUN npm install --global --omit=dev snarkjs@0.7.5

FROM runtime AS operator
ENV ZISK_HOME=/app/.zisk
COPY --from=snarkjs /usr/local/bin/node /usr/local/bin/node
COPY --from=snarkjs /usr/local/lib/node_modules/snarkjs /usr/local/lib/node_modules/snarkjs
RUN ln -s ../lib/node_modules/snarkjs/build/cli.cjs /usr/local/bin/snarkjs \
    && mkdir -p "$ZISK_HOME" \
    && chown bridge:bridge "$ZISK_HOME" \
    && test -x /usr/local/bin/snarkjs \
    && node -e "if (require('/usr/local/lib/node_modules/snarkjs/package.json').version !== '0.7.5') process.exit(1)"

FROM ${BIN_MODE} AS run
ARG BIN_MODE
ENV BIN_MODE_ENV=$BIN_MODE
COPY --from=builder /build/vectorx-$BIN_MODE /usr/local/bin
RUN chown bridge:bridge /usr/local/bin/vectorx-$BIN_MODE
USER bridge

ENTRYPOINT ["/bin/sh", "-c"]
CMD ["/usr/local/bin/vectorx-${BIN_MODE_ENV}"]
