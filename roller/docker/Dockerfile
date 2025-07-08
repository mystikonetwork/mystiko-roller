FROM rust:ubuntu-22.04
RUN apt update && \
    apt install -y curl && \
    mkdir -p /home/mystiko-miner/roller/config \
             /home/mystiko-miner/roller/data \
             /home/mystiko-miner/roller/circuits
COPY target/release/mystiko_roller /usr/local/bin/mystiko_roller
CMD ["mystiko_roller"]
