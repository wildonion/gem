FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
RUN rm core/*.rs

ADD . ./

RUN cargo build --bin bot --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /gem/target/release/bot ${APP}/bot

RUN chown -R $APP_USER:$APP_USER ${APP}

COPY ./.env ${APP}/.env
RUN USER=root mkdir ${APP}/openai-logs
RUN USER=root mkdir ${APP}/gpt-logs
RUN USER=root mkdir ${APP}/error-kind
RUN USER=root mkdir ${APP}/rate-limiter

USER $APP_USER
WORKDIR ${APP}

CMD ["./bot"]