FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
RUN rm src/*.rs

RUN USER=root echo DB_ENGINE=postgres >> .env
RUN USER=root echo DB_HOST=postgres >> .env
RUN USER=root echo DB_PORT=5432 >> .env
RUN USER=root echo ENVIRONMENT=prod >> .env
RUN USER=root echo DATABASE_URL=postgres://postgres:geDteDd0Ltg2135FJYQ6rjNYHYkGQa70@postgres/conse >> .env

ADD . ./

RUN apt install -y libpq-dev
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --bin panel --release

RUN diesel setup && diesel migration run


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