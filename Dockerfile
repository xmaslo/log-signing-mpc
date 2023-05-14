FROM rust:1.69 as base

COPY . .

RUN cargo install --path .
