FROM rust:1.69

COPY . .

FROM base as test
RUN cargo test

FROM base as install
RUN cargo install --path .
