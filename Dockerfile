# build stage
FROM rust:slim AS builder

WORKDIR /app

RUN apt-get update -y \
    && apt-get install libudev-dev pkg-config -y

COPY . .

RUN cargo build --release

# runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/xcontroller xcontroller

ENTRYPOINT ["sh", "-c", "./xcontroller ${WEBSOCKET_PORT} ${SERIAL_PORT} ${BAUDRATE} ${TEST_MODE}"]
