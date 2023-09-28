FROM ubuntu:22.04
COPY target/release/mystiko_roller /usr/local/bin/mystiko_roller
COPY tests/test_files/home/config/roller.json /home/mystiko-miner/roller/config/roller.json
CMD ["mystiko_roller"]
