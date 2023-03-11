# Rust as the base image
FROM rust:1.68 as build

WORKDIR /usr/gem
RUN USER=root mkdir gem
COPY . gem

WORKDIR /usr/gem/

RUN cargo install --path .
RUN cargo build --bin conse --release

FROM debian:buster-slim
COPY --from=build /usr/gem/target/release/conse .
COPY .env .

EXPOSE 4378

CMD ["./conse"]