FROM rust:1.69

COPY . .

RUN cargo test && cargo install --path .
