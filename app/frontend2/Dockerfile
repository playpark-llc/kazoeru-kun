FROM rust:1-slim

WORKDIR /opt/app/frontend2/yew-app

RUN apt-get update \
    && apt-get install --no-install-recommends -y libssl-dev pkg-config \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && apt update \
    && apt install -y nodejs npm

COPY ./yew-app/package.json .
COPY ./yew-app/package-lock.json .

RUN npm run setup