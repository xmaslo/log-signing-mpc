#!/bin/bash

# Start the three servers
cargo run -- 0 8000 &
server1_pid=$!
cargo run -- 1 8001 &
server2_pid=$!
cargo run -- 2 8002 &
server3_pid=$!

# Wait for servers to start
sleep 5

# Initialize the servers in the background
curl -X POST localhost:8000/init_room -d "127.0.0.1:8001,127.0.0.1:8002" &
init1_pid=$!
curl -X POST localhost:8001/init_room -d "127.0.0.1:8002,127.0.0.1:8000" &
init2_pid=$!
curl -X POST localhost:8002/init_room -d "127.0.0.1:8001,127.0.0.1:8000" &
init3_pid=$!

# Send test requests
curl -X POST localhost:8000/send_broadcast -d "{\"sender\":0,\"receiver\":2,\"body\":\"Learning rust step by step\"}"
curl -X POST localhost:8000/send_broadcast -d "{\"sender\":0,\"receiver\":1,\"body\":\"Learning rust step by step\"}"
curl -X POST localhost:8000/send_broadcast -d "{\"sender\":2,\"receiver\":0,\"body\":\"Learning rust step by step\"}"

sleep 60

kill init1_pid
kill init2_pid
kill init3_pid

# Wait for the servers to finish
kill $server1_pid
kill $server2_pid
kill $server3_pid