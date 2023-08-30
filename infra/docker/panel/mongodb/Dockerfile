# --- Build Stage ---
FROM rust:slim-bullseye as builder

ENV LD_LIBRARY_PATH=/usr/local/lib/:$LD_LIBRARY_PATH

WORKDIR /usr/src/app
COPY . /usr/src/app

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    build-essential \
    libssl-dev \
    libpq5 \
    libpq-dev \
    git \
    ca-certificates \
    tzdata && \
    rm -rf /var/lib/apt/lists/* && \
    git clone https://github.com/cossacklabs/themis.git && \
    cd themis && \
    make install && \
    cd .. && \
    cargo build --bin panel --release

# --- Runtime Stage ---
FROM debian:bullseye-slim
ARG ARCH=x86_64

ENV LD_LIBRARY_PATH=/usr/local/lib/:$LD_LIBRARY_PATH
ENV DB_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
ENV DB_USERNAME=root
ENV DB_ENGINE=mongodb
ENV DB_HOST=mongodb
ENV DB_PORT=27017
# if ENVIRONMENT was set to dev DB_USERNAME and DB_PASSWORD will be ignored
ENV ENVIRONMENT=dev 
ENV TZ=Etc/UTC
ENV APP_USER=appuser

WORKDIR /app

# copy necessary libs from the builder stage into this final stage
# cause the final binary requires linking files or .so lilke crypto
# and postgres libs to be in its path.
COPY --from=builder /usr/src/app/target/release/panel /app/panel
COPY --from=builder /usr/local/lib/lib* /usr/local/lib/
COPY --from=builder /usr/lib/${ARCH}-linux-gnu/lib* /usr/lib/${ARCH}-linux-gnu/
COPY --from=builder /lib/${ARCH}-linux-gnu/lib* /lib/${ARCH}-linux-gnu/

# granting the access for the root to create any folder at runtime
RUN USER=root chown -R root:root /app

# we must copy necessary runtime files into this stage for ./panel
COPY ./.env /app/.env
COPY ./twitter-accounts.json /app/twitter-accounts.json

USER root

EXPOSE 7442

CMD ["./panel"]