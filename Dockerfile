FROM rust:1.69 as base

COPY . .

RUN cargo build --release
