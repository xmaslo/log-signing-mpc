# MPC-Log-Signing - General Info
This project defines a standalone server that (combined with others) can be used for threshold signatures using ECDSA.

## Original Project
This repository is based on the backend of a timestamping project from the PV204 course (https://github.com/davidmaslo/timestamping-server). The original authors of the said project were:
- Dávid Maslo (me)
- Adam Hlaváček
- David Rajnoha

This project is its direct fork. From commit https://github.com/davidmaslo/timestamping-server/commit/71a01282d7b1577d11576d039573256edef9deee, I worked on this project entirely on my own.

# Deployment

1. [Bare metal](#bare-metal) (both Windows and Linux).
2. [Docker image](#docker-image).

# Bare metal

## TLS

First, you must create and provide a TLS certificate, certificate authority, and a private key.
The server will look for them in the `certs` directory. The directory must be located in the same directory as the executable.
The ca certificate lies directly in that directory and is named ca_cert.pem.
The public certificate and the private key must be located in a subdirectory named `private` and public, respectively.
The certificate and the private key must be named `cert_{server_id}.pem` and `private_key_{server_id}.pem` respectively.

For easier development usage, you can unpack the certificates in `examples/certs.zip`, which stores 9 certificates with 100 years of validity (only use them for development).

If you want to create your self-signed certificates, you can do the following:
1. Navigate into the `log-signing-mpc` directory.
2. Run:
   - Linux:
      - `./examples/certs_creation.sh`
   - Windows: First, install Git Bash (https://git-scm.com/downloads) and then run: \
     `export MSYS_NO_PATHCONV=1` \
     `./examples/certs_creation_win.sh 4`
3. Copy the `certs` directory to the one from which you will run `log-signing-mpc.exe`. Usually, it is the `log-signing-mpc\target\release`.

## Build, Run, and Test

The newest version of the compiler that can be used for this project is 1.72.0 because it contains dependencies that are no longer supported and need to be replaced.

You can download the most recent version of Rustup at https://www.rust-lang.org/tools/install}{https://www.rust-lang.org/tools/install and then downgrade to version 1.72.0 by simply running `rustup install 1.72.0` and `rustup default 1.72.0`.

### Build
1. Build Debug: `cargo build` \
   Build Release: `cargo build --release`

### Run
Note: If running in PowerShell, you might need to change the execution policy (administrator privileges required): `set-executionpolicy remotesigned`.

Navigate to `.\log-signing-mpc\scripts` and run:
1. `.\run_servers_on_localhost.ps1 ..\target\release 2 4`

The format is `.\run_servers_on_localhost.ps1 build-directory threshold number-of-parties`.

OR

Navigate to `log-signing-mpc\target\release` and run:
1. `.\log-signing-mpc.exe 1 8001 3001 2 4`
2. `.\log-signing-mpc.exe 2 8002 3002 2 4`
3. `.\log-signing-mpc.exe 3 8003 3003 2 4`
4. `.\log-signing-mpc.exe 4 8004 3004 2 4`

The format is: `.\log-signing-mpc.exe server-id HTTP-port TLS-port threshold number-of-parties`.

### Test
To run tests, follow:
1. `pip install -r .\evaluation\requirements.txt`
2. In the `tests\common`, modify the `IS_DOCKER` to be **False**.
3. In the `scripts`, to run servers: `.\run_servers_on_localhost.ps1 ..\target\release 2 4`.
4. `pytest -k "TestKeyGen24 or TestSigning24 or TestVerify24"`.
5. Make sure the previous test runs did not already generate the keys; otherwise, the first test will fail (if generated, keys are usually present in the `target\release` under names `local-shareX.json`, where X is the number of a share).

## Threshold Signature Operations

There is a more convenient way to run these operations if you are not interested in the details,
see [Alternative Way to Run TS Operations](#alternative-way-to-run-ts-operations).

### Key Generation

To generate keys, curl the */keygen* endpoint (you can download curl at https://curl.se/windows/):
1. `curl.exe -X POST localhost:8001/key_gen/1 -d "127.0.0.1:3002,127.0.0.1:3003,127.0.0.1:3004"`
2. `curl.exe -X POST localhost:8002/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3003,127.0.0.1:3004"`
3. `curl.exe -X POST localhost:8003/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3002,127.0.0.1:3004"`
4. `curl.exe -X POST localhost:8004/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3002,127.0.0.1:3003"`

The format is `curl.exe -X POST localhost:{HTTP-port}/key_gen/1 -d "{OTHER-SERVER-URLS-WITH-THEIR-TLS-PORTS}"`.

Each server will generate its keys named `local-shareX.json`, where X is the server's id (usually in `target\release` directory).

### Signing

To sign a message, curl the  */sign* endpoint (you can convert signature to hex string at https://string-functions.com/string-hex.aspx):
1. `curl.exe -X POST localhost:8001/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":2,\"url\":\"127.0.0.1:3002\"},{\"server_id\":3,\"url\":\"127.0.0.1:3003\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`
2. `curl.exe -X POST localhost:8002/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":1,\"url\":\"127.0.0.1:3001\"},{\"server_id\":3,\"url\":\"127.0.0.1:3003\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`
3. `curl.exe -X POST localhost:8003/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":1,\"url\":\"127.0.0.1:3001\"},{\"server_id\":2,\"url\":\"127.0.0.1:3002\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`

The format is `curl.exe -X POST localhost:{HTTP-port}/sign/1 -H "Content-Type: application/json" '{JSON-DATA}`.

The `JSON-DATA}` are of the following format:

```
{
    "participants":
    [
        {
            "server_id": {SERVER-ID},
            "url": {SERVER-URL}
        },
        ...
    ],
    "data_to_sign": "{SHA256-HASH-OF-DATA}",
    "timestamp": "{TIMESTAMP}"
}
```

Note: This is a PoC implementation, and as such, the synchronization of the servers is done quite poorly.
You will have to run all three curls very quickly in succession (ideally in parallel). Otherwise, the signature will fail. If you cannot do so, you can use pre-prepared Python scripts; see [Alternative Way to Run TS Operations](#alternative-way-to-run-ts-operations).

### Verification

To verify a signature, curl the  */verify* endpoint:
1. `curl.exe -X POST localhost:8001/verify -d "7b2272223a7b226375727665223a22736563703235366b31222c227363616c6172223a5b3235352c3233322c36372c33372c33372c3230342c3136322c34392c3133322c3132312c3130312c3134302c39312c3130332c3137392c37392c3135372c37302c35352c33382c3131322c31372c3130372c3133352c362c3132302c3134312c37382c3131342c3130392c3131362c3137355d7d2c2273223a7b226375727665223a22736563703235366b31222c227363616c6172223a5b36302c3137302c3134322c33312c3230332c3137322c35302c3234302c31322c3230352c3231312c32322c32312c3137302c3133362c3233372c31352c3139362c36342c39392c3231332c3135312c38322c35372c3230302c38312c37352c3136362c3234322c3233302c32302c335d7d2c227265636964223a307d,7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d,16816533390"`

The format is `curl.exe -X POST localhost:{HTTP-port}/verify -d "{SIGNATURE-AS-HEX-STRING},{SHA256-HASH-OF-DATA},{TIMESTAMP}"`.

Note: This should produce an "Invalid signature" because you have different keys.

# Docker image

## TLS
TLS certificates are generated automatically during the image creation process.

## Build, Run, and Test

### Build
Navigate to the root directory and run:
1. `docker compose build build-service-24`.

Alternatively, if you do not want to use **docker compose**, you can build it yourself:
1. Build the image: `docker build -t log-signing-mpc .`
2. Build a network: `docker network create la-net`

For more information about local networking with docker containers, follow https://docs.docker.com/network/network-tutorial-standalone/.

### Run
Navigate to the root directory and run:
1. `docker compose -f .\docker-compose-24.yml up`

OR

If you do not want to use **docker compose**, you can run it yourself:
1. Run server 1: `docker run --name la1 --network la-net --rm -p 8001:8001 -p 3001:3001 log-signing-mpc-image 1 8001 3001 2 4`
2. Run server 2: `docker run --name la2 --network la-net --rm -p 8002:8002 -p 3002:3002 log-signing-mpc-image 2 8002 3002 2 4`
3. Run server 3: `docker run --name la3 --network la-net --rm -p 8003:8003 -p 3003:3003 log-signing-mpc-image 3 8003 3003 2 4`
4. Run server 4: `docker run --name la4 --network la-net --rm -p 8004:8004 -p 3004:3004 log-signing-mpc-image 4 8004 3004 2 4`

### Test
Same as in the [Test](#test), but set `IS_DOCKER` to **True**.

## Threshold Signature Operations

### Key Generation
Same as in the [Key Generation](#key-generation), but the addresses are different:
1. `curl.exe -X POST localhost:8001/key_gen/1 -d "la2:3002,la3:3003,la4:3004"`
2. `curl.exe -X POST localhost:8002/key_gen/1 -d "la1:3001,la3:3003,la4:3004"`
3. `curl.exe -X POST localhost:8003/key_gen/1 -d "la1:3001,la2:3002,la4:3004"`
4. `curl.exe -X POST localhost:8004/key_gen/1 -d "la1:3001,la2:3002,la3:3003"`

### Signing
Same as in the [Signing](#signing), but the addresses are different:
1. `curl.exe -X POST localhost:8001/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":2,\"url\":\"la2:3002\"},{\"server_id\":3,\"url\":\"la3:3003\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`
2. `curl.exe -X POST localhost:8002/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":1,\"url\":\"la1:3001\"},{\"server_id\":3,\"url\":\"la3:3003\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`
3. `curl.exe -X POST localhost:8003/sign/1 -H "Content-Type: application/json" -d '{\"participants\":[{\"server_id\":1,\"url\":\"la1:3001\"},{\"server_id\":2,\"url\":\"la2:3002\"}],\"data_to_sign\":\"7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d\",\"timestamp\":\"16816533390\"}'`

### Verification
The same as in the [Verification](#verification).

# Alternative Way to Run TS Operations
Navigate into the `evaluation\simple_operations` and run: \
*Key generation:* `python key_generation.py <number_of_nodes>`, \
*Signing:* `python signing.py <threshold> <data_to_sign>`, \
*Verification:* `python verification.py <server_id> <signed_data> <timestamp> <signature_in_hex>`.

The *key\_generation.py* takes the number of servers a scheme has and generates keys. The *signing.py* takes the
threshold of the scheme with data that you want to sign and prints to the standard output the signature in hex and the
timestamp used. The *verification.py* takes the server_id on which you want to do verification, the data you signed,
and the timestamp with the signature provided by the signing.py script. It Then prints whether the signature is **valid** or **invalid**.

Examples of usage: \
`python .\key_generation.py 4` \
`python .\signing.py 2 data123` \
`python .\verification.py 4 data123 1702548777 7b2272223a7b226375727665223a22736563703235366b31222c227363616c6172223a5b34352c3230382c3132302c3231302c3131352c38352c3133392c31312c3133362c3137322c31312c3231392c3139312c3130342c3136332c3230372c31352c38332c37372c3134302c3232392c372c3233332c3133322c3233312c3136352c3138322c31312c3132392c38372c3130342c36315d7d2c2273223a7b226375727665223a22736563703235366b31222c227363616c6172223a5b3132372c3134372c3139332c33392c3130392c3130392c34312c34352c39342c33372c3134362c3132372c3131382c31342c37362c39362c372c3136352c36382c3133322c3131312c3132362c3139352c36372c392c3137392c3133362c36362c3137312c3131372c35392c375d7d2c227265636964223a307d`

# Evaluation
1. Add your project root directory into the PYTHONPATH: `$env:PYTHONPATH = "D:\log-signing-mpc;$env:PYTHONPATH"`.
2. Navigate to the `evaluation\performance`.
3. Run the performance benchmark of your choice: \
   **Key generation:** `python key_generation.py <number_of_nodes> <number_of_trials> <path_to_generated_keys>`, \
   **Signing:** `python signing.py <threshold> <test_type> <log_count> <number_of_trials>`, \
   **Verification:** `python verification.py <threshold> <number_of_signatures_to_verify> <number_of_trials>`.

For **key generation**, you have to specify the number of servers your schemes use, the number of trials of tests you want
to run, and where the keys will be generated since they need to be deleted to generate them repeatedly (Note: this
currently, it only works on non-docker deployment because I did not find a good way to remove keys from the containers).

For **signing**, you have to specify the threshold of the scheme, test type, number of logs to sign, and the number of trials.
*Test_type* can be either *order* or *parallel*. If it is *order*, *number_of_logs* specifies the number of logs that
will be signed in order in a single trial, where *number_of_trials* specifies their count. If it is *parallel*, the *number_of_logs*
specifies how many logs will be sent in parallel, and *number_of_trials* specifies how many trials of that test you want
to do.

For **verification**, you have to specify the threshold of the scheme, the number of signatures to verify in a single trial,
and number of trials you want to do.

The scripts will then present to you the intermediate results and the final average. I would just like to point out that
signing is not 100% perfect if you sign many logs in parallel. This is because that would require robust synchronization
between servers, which is rather complicated to get right.

Examples: \
`python .\key_generation.py 4 5 ..\..\target\release` \
`python .\signing.py 2 order 1 10`\
`python .\signing.py 2 parallel 5 2` \
`python .\verification.py 2 100 5`.


# Useful Commands
1. Run unit-tests inside docker: `docker compose run unit-tests`.
2. Static analysis: `cargo clippy`
3. Test project with command line output: `cargo test -- --nocapture`.
4. Run the image interactively with bash: `docker run -it --entrypoint bash log-signing-mpc-image`
5. Do not suppress docker output: `docker compose build build-service --progress=plain --no-cache`
