FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
COPY ./addrs.json ./addrs.json
RUN rm src/*.rs

RUN USER=root echo DB_ENGINE=mongodb >> .env
RUN USER=root echo DB_HOST=mongodb >> .env
RUN USER=root echo DB_PORT=7441 >> .env
RUN USER=root echo ENVIRONMENT=dev >> .env

ADD . ./

RUN cargo build --bin conse --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 7438

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /gem/target/release/conse ${APP}/conse

RUN chown -R $APP_USER:$APP_USER ${APP}

COPY ./.env ${APP}/.env
COPY ./addrs.json ${APP}/addrs.json

RUN USER=root mkdir ${APP}/assets

USER $APP_USER
WORKDIR ${APP}

CMD ["./conse"]