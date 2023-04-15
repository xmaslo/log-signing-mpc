# PV204-Project
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
1. `.\timestamping-server.exe 1 8000`
2. `.\timestamping-server.exe 2 8001`
3. `.\timestamping-server.exe 3 8002`


## Key Generation

To generate keys, curl the */keygen* endpoint (you can download curl at https://curl.se/windows/):
1. `curl.exe -X POST localhost:8000/key_gen/1 -d "127.0.0.1:8001,127.0.0.1:8002"`
2. `curl.exe -X POST localhost:8001/key_gen/1 -d "127.0.0.1:8002,127.0.0.1:8000"`
3. `curl.exe -X POST localhost:8002/key_gen/1 -d "127.0.0.1:8001,127.0.0.1:8000"`

## Signing

To sign a message, curl the  */sign* endpoint:
1. `curl.exe -X POST localhost:8000/sign/2 -d "2,127.0.0.1:8001,sign_this_data,1681399682"`
2. `curl.exe -X POST localhost:8001/sign/2 -d "1,127.0.0.1:8000,sign_this_data,1681399682"`

Format is -d `"other_party_id,other_party_address,data_to_sign,unix_seconds_timestamp"`.

You can find current timestamp at https://www.epochconverter.com/.

NOTE: Sometimes, the servers just get stuck. In that case, re-run the curls.

## Verification

1. `curl.exe -X POST localhost:8000/verify -d '{\"r\":{\"curve\":\"secp256k1\",\"scalar\":[139,102,147,178,33,120,45,252,48,92,163,170,108,234,164,3,97,83,6,55,114,62,241,157,76,109,9,244,168,181,150,216]},\"s\":{\"curve\":\"secp256k1\",\"scalar\":[11,58,152,231,158,44,193,39,98,99,74,78,217,171,197,230,236,142,171,2,199,82,171,190,237,19,171,10,224,216,233,56]},\"recid\":0};sign_this_data;1681559007'`
    - Note that escaping quotes is only necessary on Windows

Format is -D "signature_output;signed_data;timestamp".

## Static Analysis

Execute `cargo clippy`

## Example Running with Cargo Run For Debugging Purposes
- `cargo run --example gg20_sm_manager --no-default-features --features curv-kzen/num-bigint`
- `cargo run --example gg20_keygen --no-default-features --features curv-kzen/num-bigint -- -t 1 -n 3 -i 1 --output local-share1.json`
- `cargo run --example gg20_signing --no-default-features --features curv-kzen/num-bigint -- -p 1,2 -d "hello" -l local-share1.json`

## Cargo Test With Command Line Output
- `cargo test -- --nocapture`

## Signing
gg_20 is a demonstration of https://eprint.iacr.org/2020/540.pdf implementation.

### Connection
First, the application joins_computation() with the server on "http://localhost:8000/" in "default-signing" room. By joining computation, it creates an http client that is issued a unique id (read from local_share json file) and creates two streams:
- incoming: filters messages addressed to someone else based on index
- outgoing: creates string from json and brodcasts it to all clients

### Offline Stage
With its index, parties specified for signing (for some reason, we need to tell it which parties should sign, like party 1,2), and local_share, offline stage can be completed. Offline stage does not mean that no communication is needed between parties, it means that message to be signed does not need to be known. If there are k rounds, then k - 1 rounds are computed in this stage.

### Online Stage
Now, with a message to be signed known, only a single round needs to be executed by parties (they need to communicate with each other) to produce a valid signature. Each participating party computes a partial signature and broadcasts it. Each party then takes partial signatures computed by other parties and using SigManual::complete() function computes the resulting signature.
