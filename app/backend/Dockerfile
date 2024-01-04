FROM rust:alpine3.18
RUN apk add libc-dev --no-cache
WORKDIR /opt/app/backend
COPY Cargo.toml .
COPY Cargo.lock .
COPY src .
CMD ["cargo", "run"]
