FROM rust:1.77-slim-buster
# FROM rust:1.79.0-alpine3.20
WORKDIR /opt/app/backend
COPY .env .
COPY .cargo .
COPY Cargo.toml .
COPY Cargo.lock .
COPY src .
COPY prisma-cli .
COPY prisma .
COPY entrypoint.sh /
RUN apt-get -y update &&\
  apt-get -y --no-install-recommends install pkg-config libssl-dev &&\
  rm -rf /var/lib/apt/lists/ &&\
  cargo build &&\
  cargo install cargo-watch &&\
  chmod u+x /entrypoint.sh
# RUN apk update &&\
#   apk add --update --no-cache openssl openssl-dev openssl-libs-static pkgconf musl-dev gcc &&\
#   cargo build &&\
#   chmod u+x /entrypoint.sh
CMD ["/bin/sh","-c","/entrypoint.sh"]
