# MPC-Log-Signing
This project defines a standalone server that (combined with others) can be used for multi-party computation using ECDSA.

## Original Project
This repository is based on the backend of a timestamping project from PV204 course (https://github.com/davidmaslo/timestamping-server). Original authors of said project were:
- Dávid Maslo (me)
- Adam Hlaváček
- David Rajnoha

This project is its direct fork. From commit https://github.com/davidmaslo/timestamping-server/commit/71a01282d7b1577d11576d039573256edef9deee, I worked on this project completely on my own.

## How to Build and Run Project

To build and run the LA servers, there are two options:
1. As separate docker containers ([Build and Run Servers inside Docker](#build-and-run-servers-inside-docker)).
2. Direct compilation on your machine ([Build and Run Servers on Bare Metal](#build-and-run-servers-on-bare-metal)).

## TLS

First, you need to create and provide a TLS certificate, certificate authority, and a private key.
The server will look for them in the `certs` directory. The directory must be located in the same directory as the executable.
The ca certificate lies directly in that directory and is named ca_cert.pem.
The public certificate and the private key must be located in a subdirectory named `private` and public respectively.
The certificate and the private key must be named `cert_{server_id}.pem` and `private_key_{server_id}.pem` respectively.

For easier development usage, you can unpack the certificates stored in `examples/certs.zip` or run the `certs_creation.sh` to
create your own self-signed certificates.


## Implemented Features
1. [Key Generation](#key-generation).
2. [Log Signing](#log-signing).
3. [Log Signature Verification](#log-signature-verification).

## Build and Run Servers inside Docker

### Run Servers Manually
This is a step-by-step guide for better understanding.
1. Build image from a dockerfile: `docker build -t log-signing-mpc .`
2. Create local network for servers: `docker network create la-net`
3. Run server 1: `docker run --name la1 --network la-net --rm -p 8000:8000 -p 3000:3000 log-signing-mpc 1 8000 3000`
4. Run server 2: `docker run --name la2 --network la-net --rm -p 8001:8001 -p 3001:3001 log-signing-mpc 2 8001 3001`
5. Run server 3: `docker run --name la3 --network la-net --rm -p 8002:8002 -p 3002:3002 log-signing-mpc 3 8002 3002`

For more information about local networking with docker containers follow https://docs.docker.com/network/network-tutorial-standalone/.

### Docker Compose
Quickly setup servers by running docker compose.
1. Build an image of a server: `docker compose build build-service`
2. Run all three servers: `docker compose up la1 la2 la3`
3. Remove containers: `docker compose down`

## Build and Run Servers on Bare Metal

### Build and Run
1. Build Debug: `cargo build` \
   Build Release: `cargo build --release`
2. `.\log-signing-mpc.exe 1 8000 3000`
3. `.\log-signing-mpc.exe 2 8001 3001`
4. `.\log-signing-mpc.exe 3 8002 3002`

## Key Generation

To generate keys, curl the */keygen* endpoint (you can download curl at https://curl.se/windows/):

### For Docker
1. `curl.exe -X POST localhost:8000/key_gen/1 -d "la2:3001,la3:3002"`
2. `curl.exe -X POST localhost:8001/key_gen/1 -d "la3:3002,la1:3000"`
3. `curl.exe -X POST localhost:8002/key_gen/1 -d "la2:3001,la1:3000"`

### For Bare Metal

1. `curl.exe -X POST localhost:8000/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3002"`
2. `curl.exe -X POST localhost:8001/key_gen/1 -d "127.0.0.1:3002,127.0.0.1:3000"`
3. `curl.exe -X POST localhost:8002/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3000"`

## Log Signing
(TODO)

To sign a message, curl the  */sign* endpoint:
1. `curl.exe -X POST localhost:8000/sign/2 -d "2,127.0.0.1:3001,0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2,1681653339"`
2. `curl.exe -X POST localhost:8001/sign/2 -d "1,127.0.0.1:3000,0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2,1681653339"`

Format is -d "other_party_id,other_party_address,data_to_sign,unix_seconds_timestamp".

You can find current timestamp at https://www.epochconverter.com/.

NOTE: Sometimes, the servers just get stuck. In that case, re-run the curls.

## Log Signature Verification
(TODO)
1. `curl.exe -X POST localhost:8000/verify -d '{\"r\":{\"curve\":\"secp256k1\",\"scalar\":[175,82,15,51,82,255,217,105,231,6,105,23,219,149,232,160,124,193,203,209,247,19,67,187,26,191,200,126,133,46,17,141]},\"s\":{\"curve\":\"secp256k1\",\"scalar\":[55,211,225,244,240,92,231,193,163,132,214,35,9,17,228,39,57,171,8,196,5,254,175,46,206,148,252,86,249,105,212,236]},\"recid\":0};0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2;1681653339'`
   Note that escaping quotes is only necessary on Windows.

Format is -d "signature_output;signed_data_with_timestamp".

## Useful Commands
1. Run unit-tests inside docker: `docker compose run unit-tests`.
2. Static analysis: `cargo clippy`
3. Test project with command line output: `cargo test -- --nocapture`.



[//]: # (## Client start-up)

[//]: # ()
[//]: # (Included client is a standalone webpage that can be served with any HTTP&#40;S&#41;-capable server.)

[//]: # (For development purposes, one can serve the client with the Python in-build HTTP server:)

[//]: # ()
[//]: # (```bash)

[//]: # (&#40; cd web-frontend && python3 -m http.server 8080 &#41;)

[//]: # (```)

[//]: # ()
[//]: # (Then navigate to [127.0.0.1:8080]&#40;http://127.0.0.1:8080&#41;.)

[//]: # ()
[//]: # (The client also support DEBUG mode, which can be enabled by pasting)

[//]: # ()
[//]: # (```js)

[//]: # (localStorage.setItem&#40;'DEBUG', '1'&#41;)

[//]: # (```)

[//]: # ()
[//]: # (into browser's console while on page.)

[//]: # ()
[//]: # (## TLS)

[//]: # ()
[//]: # (First, you need to create and provide a TLS certificate, certificate authority, and a private key.)

[//]: # (The server will look for them in the `certs` directory. The directory must be located in the same directory as the executable.)

[//]: # (The ca certificate lies directly in that directory and is named ca_cert.pem.)

[//]: # (The public certificate and the private key must be located in a subdirectory named `private` and public respectively.)

[//]: # (The certificate and the private key must be named `cert_{server_id}.pem` and `private_key_{server_id}.pem` respectively.)

[//]: # ()
[//]: # (For easier development usage, you can unpack the certificates stored in `examples/certs.zip` or run the `certs_creation.sh` to)

[//]: # (create your own self-signed certificates.)

[//]: # ()
[//]: # (## Server Setup on Linux)

[//]: # ()
[//]: # (Run `keygen_example.sh` script. For subsequent runs, a `start-stop.sh` script is available:)

[//]: # (```bash)

[//]: # (./start-stop.sh start 1  # starts first server)

[//]: # (./start-stop.sh stop 1 # stops first server)

[//]: # (./start-stop.sh restart 1 # restarts first server)

[//]: # (./start-stop.sh start all # starts all servers)

[//]: # (```)

[//]: # ()
[//]: # (## Example Running with Cargo Run For Debugging Purposes)

[//]: # (- cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint)

[//]: # (- cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json)

[//]: # (- cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json)

[//]: # ()
[//]: # (## Static Analysis)

[//]: # ()
[//]: # (Execute `cargo clippy`)

[//]: # ()
[//]: # (## Example Running with Cargo Run For Debugging Purposes)

[//]: # (- `cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint`)

[//]: # (- `cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json`)

[//]: # (- `cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json`)

[//]: # ()
[//]: # (## Cargo Test With Command Line Output)

[//]: # (- `cargo test -- --nocapture`)
