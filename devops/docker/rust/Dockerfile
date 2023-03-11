# Rust as the base image
FROM rust:1.68 as build

WORKDIR /usr/gem
COPY . .

RUN cargo install --path .
RUN cargo build --bin conse --release

FROM debian:buster-slim
COPY --from=build /conse/target/release/conse /usr/gem/conse

EXPOSE 4378

CMD ["/usr/gem/conse"]