FROM rust:1.69

WORKDIR /usr/src/log-signing-mpc
COPY . .

RUN rm -rf certs && ./examples/certs_creation.sh
RUN cargo build --release && cp -r certs ./target/release/

ENTRYPOINT ["/usr/src/log-signing-mpc/target/release/log-signing-mpc"]
