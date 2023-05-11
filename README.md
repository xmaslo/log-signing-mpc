# MPC-Log-Signing
Trusted timestamping server with threshold signing key

## Team Members
- Dávid Maslo
- Adam Hlaváček
- David Rajnoha

## Installation
1. `git@github.com:davidmaslo/timestamping-server.git` (or use https instead)
2. `cd timestamping-server`
3. `cargo build`

## Client start-up

Included client is a standalone webpage that can be served with any HTTP(S)-capable server.
For development purposes, one can serve the client with the Python in-build HTTP server:

```bash
( cd web-frontend && python3 -m http.server 8080 )
```

Then navigate to [127.0.0.1:8080](http://127.0.0.1:8080).

The client also support DEBUG mode, which can be enabled by pasting

```js
localStorage.setItem('DEBUG', '1')
```

into browser's console while on page.

## TLS

First, you need to create and provide a TLS certificate, certificate authority, and a private key.
The server will look for them in the `certs` directory. The directory must be located in the same directory as the executable.
The ca certificate lies directly in that directory and is named ca_cert.pem.
The public certificate and the private key must be located in a subdirectory named `private` and public respectively.
The certificate and the private key must be named `cert_{server_id}.pem` and `private_key_{server_id}.pem` respectively.

For easier development usage, you can unpack the certificates stored in `examples/certs.zip` or run the `certs_creation.sh` to
create your own self-signed certificates.

## Server Setup

### Linux
Run `keygen_example.sh` script. For subsequent runs, a `start-stop.sh` script is available:
```bash
./start-stop.sh start 1  # starts first server
./start-stop.sh stop 1 # stops first server
./start-stop.sh restart 1 # restarts first server
./start-stop.sh start all # starts all servers
```

### Windows
Run all three timestamping servers as follows:

1. `.\timestamping-server.exe 1 8000 3000`
2. `.\timestamping-server.exe 2 8001 3001`
3. `.\timestamping-server.exe 3 8002 3002`

## Key Generation

To generate keys, curl the */keygen* endpoint (you can download curl at https://curl.se/windows/):
1. `curl.exe -X POST localhost:8000/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3002"`
2. `curl.exe -X POST localhost:8001/key_gen/1 -d "127.0.0.1:3002,127.0.0.1:3000"`
3. `curl.exe -X POST localhost:8002/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3000"`

NOTE: On our Windows machine, the key generation does not work with release version of our application. If that is the case for you too, generate keys on debug one, and you can continue to use the release one. But since these are servers anyway, we target them on Linux platform. There everything should work.

## Signing
To sign a message, curl the  */sign* endpoint:
1. `curl.exe -X POST localhost:8000/sign/2 -d "2,127.0.0.1:3001,0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2,1681653339"`
2. `curl.exe -X POST localhost:8001/sign/2 -d "1,127.0.0.1:3000,0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2,1681653339"`

Format is -d "other_party_id,other_party_address,data_to_sign,unix_seconds_timestamp".

You can find current timestamp at https://www.epochconverter.com/.

NOTE: Sometimes, the servers just get stuck. In that case, re-run the curls.

## Verification
1. `curl.exe -X POST localhost:8000/verify -d '{\"r\":{\"curve\":\"secp256k1\",\"scalar\":[175,82,15,51,82,255,217,105,231,6,105,23,219,149,232,160,124,193,203,209,247,19,67,187,26,191,200,126,133,46,17,141]},\"s\":{\"curve\":\"secp256k1\",\"scalar\":[55,211,225,244,240,92,231,193,163,132,214,35,9,17,228,39,57,171,8,196,5,254,175,46,206,148,252,86,249,105,212,236]},\"recid\":0};0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2;1681653339'`
Note that escaping quotes is only necessary on Windows.
   
Format is -d "signature_output;signed_data_with_timestamp".

## Static Analysis
cargo clippy

## Example Running with Cargo Run For Debugging Purposes
- cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint
- cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json
- cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json

## Cargo Test With Command Line Output
- cargo test -- --nocapture

## Static Analysis

Execute `cargo clippy`

## Example Running with Cargo Run For Debugging Purposes
- `cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint`
- `cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json`
- `cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json`

## Cargo Test With Command Line Output
- `cargo test -- --nocapture`
