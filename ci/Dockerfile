FROM rustlang/rust:nightly-buster-slim

RUN apt-get update
RUN apt-get -y install libssl-dev pkg-config cmake zlib1g-dev
RUN cargo install cargo-tarpaulin