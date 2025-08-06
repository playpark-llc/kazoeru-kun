FROM rust:1.85.0-slim-bullseye
RUN apt-get -y update &&\
  apt-get -y --no-install-recommends install pkg-config libssl-dev &&\
  cargo install sea-orm-cli@1.0.1  cargo-watch  cargo-make &&\
  rustup component add rustfmt &&\
  rm -rf /var/lib/apt/lists/
WORKDIR /opt/app/backend
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo fetch

COPY .env .
COPY migration  ./migration
COPY entrypoint.sh /

RUN cargo build &&\
  chmod u+x /entrypoint.sh
CMD ["/bin/sh","-c","/entrypoint.sh"]
