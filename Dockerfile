FROM rust:1.69

WORKDIR /usr/src/log-signing-mpc
COPY . .

RUN sed 's/^M$//' ./examples/certs_creation.sh && ./examples/certs_creation.sh  # convert CRLF to LF and run
RUN cargo build --release && cp -r certs ./target/release/

ENTRYPOINT ["/usr/src/log-signing-mpc/target/release/log-signing-mpc"]
