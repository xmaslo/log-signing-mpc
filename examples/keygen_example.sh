#!/bin/bash
set -xeuo pipefail

# First generate the SSL certificates
bash examples/certs_creation.sh

# Start the three servers
cargo run -- 1 8000 3000 &
server1_pid=$!
cargo run -- 2 8001 3001 &
server2_pid=$!
cargo run -- 3 8002 3002 &
server3_pid=$!

# Wait for servers to start
sleep 5

# Initialize the servers in the background
curl -X POST localhost:8000/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3002" &
curl -X POST localhost:8001/key_gen/1 -d "127.0.0.1:3002,127.0.0.1:3000" &
curl -X POST localhost:8002/key_gen/1 -d "127.0.0.1:3001,127.0.0.1:3000" &

sleep 60

curl -X POST localhost:8000/sign/2 -d "2,127.0.0.1:3001,sign_this_data,$(date +%s)" &
curl -X POST localhost:8001/sign/2 -d "1,127.0.0.1:3000,sign_this_data,$(date +%s)" &

sleep 100

# Wait for the servers to finish
kill $server1_pid
kill $server2_pid
kill $server3_pid
