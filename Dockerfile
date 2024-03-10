FROM rust:1.67.1 AS build

WORKDIR /build
COPY src src
COPY target target
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=build /build/target/release/minecraft-server-sentry.exe /app/minecraft-server-sentry.exe

ENTRYPOINT ["minecraft-server-sentry.exe"]