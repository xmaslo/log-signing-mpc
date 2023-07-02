#!/bin/bash

mkdir certs certs/public certs/private

# Generate CA key pair and self-signed certificate
openssl req -new -x509 -days 30 -newkey rsa:4096 -nodes -keyout "certs/ca_key.pem" \
 -out "certs/ca_cert.pem" \
 -subj "/C=CZ/ST=JMK/L=NA/O=pv204 Issuer/CN=127.0.0.1"

# Generate client key pairs and CSRs
for i in {1..3}; do
  openssl req -new -newkey rsa:4096 -nodes -keyout "certs/private/private_key_${i}.pem" \
   -config "examples/san.cnf" \
   -out "certs/public/csr_${i}.pem"
done

# Sign client CSRs with CA certificate
for i in {1..3}; do
  openssl x509 -req -in "certs/public/csr_${i}.pem" -CA "certs/ca_cert.pem" -CAkey "certs/ca_key.pem" \
    -CAcreateserial \
    -out "certs/public/cert_${i}.pem" -days 30 \
    -extfile <(printf "subjectAltName=DNS:localhost,IP:127.0.0.1")
done

for i in {1..3}; do
  cat "certs/public/cert_${i}.pem" "certs/private/private_key_${i}.pem" > "certs/private/cert_and_key_${i}.pem"
done
