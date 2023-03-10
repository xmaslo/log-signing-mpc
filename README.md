# PV204-Project
Trusted timestamping server with threshold signing key

## Team Members
- Dávid Maslo
- Adam Hlaváček
- David Rajnoha

## Installation
1. git@github.com:davidmaslo/timestamping-server.git (or use https instead)
2. cd timestamping-server
3. cargo run

## Static Analysis
cargo clippy

## Example Running with Cargo Run For Debugging Purposes
- cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint
- cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json
- cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json

## Cargo Test With Command Line Output
- cargo test -- --nocapture

## Signing
My understanding is that gg_20 is a demonstration of https://eprint.iacr.org/2020/540.pdf implementation.

### Connection
First, the application joins_computation() with the server on "http://localhost:8000/" in "default-signing" room. By joining computation, it creates an http client that is issued a unique id (read from local_share json file) and creates two streams:
- incoming: filters messages addressed to someone else based on index
- outgoing: creates string from json and brodcasts it to all clients

### Offline Stage
With its index, parties specified for signing (for some reason, we need to tell it which parties should sign, like party 1,2), and local_share, offline stage can be completed. Offline stage does not mean that no communication is needed between parties, it means that message to be signed does not need to be known. If there are k rounds, then k - 1 rounds are computed in this stage.

### Online Stage
Now, with a message to be signed known, only a single round needs to be executed by parties (they need to communicate with each other) to produce a valid signature. Each participating party computes a partial signature and broadcasts it. Each party then takes partial signatures computed by other parties and using SigManual::complete() function computes the resulting signature.
