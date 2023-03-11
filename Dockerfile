# Rust as the base image
FROM rust:1.68 as build

RUN USER=root mkdir gem

WORKDIR /gem
COPY . .

RUN cargo install --path .
RUN cargo build --bin conse --release

FROM debian:buster-slim
COPY ./target/release/conse .

EXPOSE 7438

CMD ["./conse"]