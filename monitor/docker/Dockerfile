FROM rust:slim

RUN apt update
RUN apt install -y curl

COPY target/release/mystiko_roller_monitor /usr/local/bin/mystiko_roller_monitor
EXPOSE 21828
CMD ["mystiko_roller_monitor"]