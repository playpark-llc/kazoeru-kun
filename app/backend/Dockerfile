FROM rust:1.76-slim-buster
WORKDIR /opt/app/backend
COPY .env .
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
  chmod u+x /entrypoint.sh
CMD ["/bin/sh","-c","/entrypoint.sh"]
