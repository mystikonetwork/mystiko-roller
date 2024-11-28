FROM rust:slim
RUN apt update && \
    apt install -y curl && \
    mkdir -p /home/mystiko-miner/roller/config \
             /home/mystiko-miner/roller/data \
             /home/mystiko-miner/roller/circuits
COPY target/release/mystiko_monitor_rollup /usr/local/bin/mystiko_monitor_rollup
EXPOSE 21829
CMD ["mystiko_monitor_rollup"]