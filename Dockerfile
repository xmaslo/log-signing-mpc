FROM rust:1.72

WORKDIR /usr/src/log-signing-mpc
COPY . .

RUN cargo build --release

ENTRYPOINT ["/usr/src/log-signing-mpc/target/release/log-signing-mpc"]
