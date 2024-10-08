FROM rust:1.77-slim-buster
# FROM rust:1.79.0-alpine3.20
RUN apt-get -y update &&\
  apt-get -y --no-install-recommends install pkg-config libssl-dev &&\
  rm -rf /var/lib/apt/lists/
RUN cargo install sea-orm-cli@1.0.0-rc.5 &&\
  cargo install cargo-watch &&\
  cargo install cargo-make &&\
  rustup component add rustfmt
WORKDIR /opt/app/backend
COPY .env .
COPY Cargo.toml .
COPY Cargo.lock .
COPY migration  ./migration
COPY src ./src
COPY entrypoint.sh /
RUN cargo build &&\
  chmod u+x /entrypoint.sh
CMD ["/bin/sh","-c","/entrypoint.sh"]
