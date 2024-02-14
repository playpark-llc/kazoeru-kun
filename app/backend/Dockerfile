FROM rust:1.76-slim-buster
WORKDIR /opt/app/backend
RUN mkdir -p prisma-cli/src
COPY .env .
COPY Cargo.toml .
COPY Cargo.lock .
COPY src .
COPY prisma-cli/Cargo.toml ./prisma-cli
COPY prisma-cli/Cargo.lock ./prisma-cli
COPY prisma-cli/src/ ./prisma-cli/src
COPY prisma ./prisma
COPY entrypoint.sh /
RUN apt-get -y update &&\
  apt-get -y install pkg-config libssl-dev &&\
  chmod u+x /entrypoint.sh
CMD ["/bin/sh","-c","/entrypoint.sh"]
