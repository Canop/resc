FROM rust:latest as builder
WORKDIR /usr/src/resc
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl
COPY --from=builder /usr/local/cargo/bin/resc /usr/local/bin/resc

RUN mkdir /usr/local/resc
VOLUME /usr/local/resc

ENTRYPOINT ["resc"]