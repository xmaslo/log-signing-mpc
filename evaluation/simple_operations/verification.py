import sys
import asyncio
from evaluation.setup import *
from evaluation.utils.endpoint_triggers import trigger_verify_endpoint

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: python signing.py <server_id> <signed_data> <timestamp> <signature_in_hex>")
        sys.exit(1)

    server_id = int(sys.argv[1])
    signed_data = sys.argv[2]
    timestamp = int(sys.argv[3])
    signature = bytes.fromhex(sys.argv[4])

    outside_ports = get_ports(server_id, 8000)

    response = asyncio.run(
        trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[server_id - 1]}",
                                signed_data,
                                signature,
                                timestamp)
    )

    print(response[1].decode('utf-8'))
