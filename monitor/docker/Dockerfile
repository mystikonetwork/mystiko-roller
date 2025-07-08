FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --bin mystiko_roller_monitor

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/mystiko_roller_monitor /usr/local/bin/mystiko_roller_monitor

EXPOSE 21828
CMD ["mystiko_roller_monitor"]