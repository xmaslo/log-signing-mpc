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
- cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json \
- cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json
