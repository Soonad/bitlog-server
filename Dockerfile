FROM rustlang/rust:nightly-buster-slim AS builder

# create a new empty shell project
RUN USER=root cargo new --bin /app
WORKDIR /app

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm -rf src

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/bitlog_server*
RUN cargo build --release

FROM debian:buster-slim

RUN mkdir /app
WORKDIR /app

COPY --from=builder /app/target/release/bitlog-server /app/bitlog-server

CMD ["/app/bitlog-server"]