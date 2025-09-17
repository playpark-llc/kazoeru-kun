# Default to development Dockerfile
# For production, use Dockerfile.prod
FROM rust:1.83.0-slim-bullseye

RUN apt-get -y update && \
  apt-get -y --no-install-recommends install pkg-config libssl-dev && \
  cargo install --locked sea-orm-cli@1.1.0 && \
  cargo install cargo-watch cargo-make && \
  rustup component add rustfmt && \
  rm -rf /var/lib/apt/lists/

WORKDIR /opt/app/backend

COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY src ./src
RUN cargo fetch

COPY .env .
COPY migration  ./migration
COPY entrypoint.sh /

RUN cargo build && \
  chmod u+x /entrypoint.sh

CMD ["/bin/sh","-c","/entrypoint.sh"]