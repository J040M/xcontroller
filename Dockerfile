# First stage: webserver (Nginx)
FROM nginx:alpine AS webserver

# Install tools to download and extract the zip file
RUN apk update && apk add --no-cache wget unzip

# Set the working directory and download the website zip
WORKDIR /tmp

RUN wget -O dist.zip https://github.com/J040M/xcontroller-app/releases/download/v0.3.0/dist.zip \
    && mkdir -p /usr/share/nginx/html \
    && unzip dist.zip -d /usr/share/nginx/html \
    && mv /usr/share/nginx/html/dist/* /usr/share/nginx/html/ \
    && rm -rf /usr/share/nginx/html/dist \
    && rm dist.zip \
    && apk del wget unzip

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

# Install dependencies (for xcontroller)
RUN apt-get update -y \
    && apt-get install -y nginx \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the built Rust application from the builder stage
COPY --from=builder /app/target/release/xcontroller xcontroller

# Copy the website files from the webserver stage
COPY --from=webserver /usr/share/nginx/html /usr/share/nginx/html

# Expose ports for both websocket and web server
EXPOSE 9002 80

# Start Nginx and xcontroller
ENTRYPOINT ["sh", "-c", "nginx -g 'daemon off;' & ./xcontroller ${WEBSOCKET_PORT} ${SERIAL_PORT} ${BAUDRATE} ${TEST_MODE}"]
