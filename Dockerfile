FROM rust:1.69

WORKDIR /usr/src/log-signing-mpc
COPY . .

RUN cargo build --release
RUN unzip examples/certs.zip && cp -r certs ./target/release/

ENTRYPOINT ["/usr/src/log-signing-mpc/target/release/log-signing-mpc"]
