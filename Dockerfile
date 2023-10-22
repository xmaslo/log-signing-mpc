FROM rust:1.72
ARG N_OF_PARTIES

WORKDIR /usr/src/log-signing-mpc
COPY . .

# convert CRLF to LF and run
RUN sed 's/^M$//' ./examples/certs_creation.sh && ./examples/certs_creation.sh $N_OF_PARTIES
RUN cargo build --release && cp -r certs ./target/release/

ENTRYPOINT ["/usr/src/log-signing-mpc/target/release/log-signing-mpc"]
