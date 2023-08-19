FROM rust:latest as builder

RUN USER=root cargo new --bin gem
WORKDIR ./gem
COPY ./Cargo.toml ./Cargo.toml
COPY ./.env ./.env 
COPY ./twitter-accounts.json ./twitter-accounts.json
RUN rm src/*.rs

ADD . ./

RUN apt-get update \
    && apt-get install libpq5 -y \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    libpq5 \
    postgresql -y

RUN USER=root apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

RUN USER=root git clone https://github.com/cossacklabs/themis.git \
    && cd themis \
    && make install


RUN cargo build --bin panel --release

# =====================================
# =====================================
# we're using bullseye-slim since it has glibc version glibc 2.31 
FROM debian:bullseye-slim
ARG APP=/usr/src/app
ARG ARCH=x86_64

ENV LD_LIBRARY_PATH=/usr/local/lib/:$LD_LIBRARY_PATH
ENV DB_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
ENV DB_USERNAME=root
ENV DB_ENGINE=mongodb
ENV DB_HOST=mongodb
ENV DB_PORT=27017
# if ENVIRONMENT was set to dev DB_USERNAME and DB_PASSWORD will be ignored
ENV ENVIRONMENT=dev 

EXPOSE 7442

ENV TZ=Etc/UTC \
    APP_USER=appuser


RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && adduser $APP_USER sudo \
    && mkdir -p ${APP}

# - Copy `libpq` dependencies into the image (Required by diesel)
COPY --from=builder /gem/target/release/panel ${APP}/panel
COPY --from=builder /usr/local/lib/libthemis.so.0 /usr/local/lib/libthemis.so.0
COPY --from=builder /usr/local/lib/lib* /usr/local/lib/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libpq.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libgssapi_krb5.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libldap_r-2.4.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libkrb5.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libk5crypto.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libkrb5support.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/liblber-2.4.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libsasl2.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libgnutls.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libp11-kit.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libidn2.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libunistring.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libtasn1.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libnettle.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libhogweed.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libgmp.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libffi.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/libssl* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/lib* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /lib/${ARCH}-linux-gnu/libcom_err.so* /lib/${ARCH}-linux-gnu/
COPY --from=builder /lib/${ARCH}-linux-gnu/libkeyutils.so* /lib/${ARCH}-linux-gnu/

RUN chown -R $APP_USER:$APP_USER ${APP}
RUN chmod -R 777 ${APP}

COPY ./.env ${APP}/.env
COPY ./twitter-accounts.json ${APP}/twitter-accounts.json

RUN USER=root mkdir ${APP}/logs
RUN USER=root chmod -R 777 ${APP}/logs
RUN USER=root mkdir ${APP}/logs/error-kind
RUN USER=root chmod -R 777 ${APP}/logs/error-kind

RUN USER=root mkdir ${APP}/assets
RUN USER=root chmod -R 777 ${APP}/assets

USER $APP_USER
WORKDIR ${APP}

CMD ["./panel"]