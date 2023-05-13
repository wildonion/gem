FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
COPY ./nfts.json ./nfts.json
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN cargo build --bin panel --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 7442

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /gem/target/release/panel ${APP}/panel

RUN chown -R $APP_USER:$APP_USER ${APP}

COPY ./.env ${APP}/.env

USER $APP_USER
WORKDIR ${APP}

CMD ["./panel"]