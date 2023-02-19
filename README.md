# PV204-Project
Trusted timestamping server with threshold signing key

## Team Members
- Dávid Maslo
- Adam Hlaváček
- David Rajnoha

## Installation
1. git clone --recurse-submodules git@github.com:xmaslo/PV204-Project.git (or use https instead)

If some sort of error occurs:
1. cd multi-party-ecdsa
2. cargo build --release --examples --no-default-features --features curv-kzen/num-bigint
