FROM rust AS builder
WORKDIR /app
COPY Cargo.lock /app
COPY Cargo.toml /app
COPY src /app/src
RUN cargo build -r

FROM debian:stable-slim
WORKDIR /app
RUN apt update && apt install libsqlite3-0 && rm -rf /var/lib/apt/lists/*
COPY --from=builder --chmod=555 /app/target/release/discord_bot /app/discord_bot
CMD [ "./discord_bot" ]