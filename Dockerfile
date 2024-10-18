FROM rust AS builder
WORKDIR /app

RUN \
  --mount=type=bind,src=.,target=. \
  --mount=type=cache,target=/usr/local/cargo/registry \
  cargo build --release --target-dir=/target


# Production container
FROM debian:stable-slim
RUN apt update && apt install libsqlite3-0 && rm -rf /var/lib/apt/lists/*
RUN addgroup --system bot && useradd --system --gid bot bot
RUN mkdir -p /app/data && chown bot /app/data
USER bot
VOLUME ["/app/data/"]
COPY --from=builder --chmod=555 /target/release/discord_bot discord_bot
CMD [ "./discord_bot" ]
