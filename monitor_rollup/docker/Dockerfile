FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --bin mystiko_monitor_rollup

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /home/mystiko-miner/roller/config \
             /home/mystiko-miner/roller/data \
             /home/mystiko-miner/roller/circuits

COPY --from=builder /usr/src/app/target/release/mystiko_monitor_rollup /usr/local/bin/mystiko_monitor_rollup

EXPOSE 21829
CMD ["mystiko_monitor_rollup"]