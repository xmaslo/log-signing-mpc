#!/bin/bash

# Start the three servers
cargo run -- 1 8000 &
server1_pid=$!
cargo run -- 2 8001 &
server2_pid=$!
cargo run -- 3 8002 &
server3_pid=$!

# Wait for servers to start
sleep 5

# Initialize the servers in the background
curl -X POST localhost:8000/key_gen/1 -d "127.0.0.1:8001,127.0.0.1:8002" &
curl -X POST localhost:8001/key_gen/1 -d "127.0.0.1:8002,127.0.0.1:8000" &
curl -X POST localhost:8002/key_gen/1 -d "127.0.0.1:8001,127.0.0.1:8000" &

sleep 300

# Wait for the servers to finish
kill $server1_pid
kill $server2_pid
kill $server3_pid