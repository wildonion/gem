FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
COPY ./twitter-accounts.json ./twitter-accounts.json
RUN rm src/*.rs

ADD . ./

RUN cargo build --bin panel --release

FROM debian:buster-slim
ARG APP=/usr/src/app

ENV DB_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
ENV DB_USERNAME=postgres    
ENV DB_ENGINE=postgres
ENV DB_HOST=postgres
ENV DB_PORT=5432
ENV ENVIRONMENT=prod

RUN apt-get update \
    && apt-get install libpq5 -y \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev

RUN USER=root apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

RUN USER=root git clone https://github.com/cossacklabs/themis.git \
    && cd themis \
    && make install

EXPOSE 7442

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /gem/target/release/panel ${APP}/panel

RUN chown -R $APP_USER:$APP_USER ${APP}

COPY ./.env ${APP}/.env
COPY ./twitter-accounts.json ${APP}/twitter-accounts.json

USER $APP_USER
WORKDIR ${APP}

CMD ["./panel"]