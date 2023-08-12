FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
COPY ./addrs.json ./addrs.json
RUN rm src/*.rs

ADD . ./

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev

RUN USER=root apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

RUN USER=root git clone https://github.com/cossacklabs/themis.git \
    && cd themis \
    && make install

RUN cargo build --bin mafia --release


FROM debian:buster-slim
ARG APP=/usr/src/app

ENV MONGODB_ENGINE=mongodb
ENV MONGODB_HOST=mongodb
ENV MONGODB_PORT=27017
ENV MONGODB_USERNAME=root
ENV MONGODB_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
# if ENVIRONMENT was set to dev DB_USERNAME and DB_PASSWORD will be ignored
ENV DB_ENVIRONMENT=dev

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

EXPOSE 7438

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /gem/target/release/mafia ${APP}/mafia

RUN chown -R $APP_USER:$APP_USER ${APP}

COPY ./.env ${APP}/.env
COPY ./addrs.json ${APP}/addrs.json

RUN USER=root mkdir ${APP}/assets

USER $APP_USER
WORKDIR ${APP}

CMD ["./mafia"]