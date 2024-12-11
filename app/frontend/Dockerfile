FROM rust:1.82.0-slim

WORKDIR /opt/app/frontend2/yew-app

RUN apt update \
  && apt install --no-install-recommends -y libssl-dev pkg-config nodejs npm \
  && apt clean \
  && rm -rf /var/lib/apt/lists/*

COPY ./yew-app/package.json .
COPY ./yew-app/package-lock.json .
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

RUN npm install
