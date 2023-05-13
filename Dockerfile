FROM rust:1.69 as base

COPY . .

FROM base as test
RUN cargo test

FROM base as install
RUN cargo install --path .
