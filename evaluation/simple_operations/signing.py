import sys
import asyncio
from evaluation.utils.signatures import get_signature
from evaluation.utils.common import get_current_timestamp
from evaluation.setup import *

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python signing.py <threshold> <data_to_sign>")
        sys.exit(1)

    threshold = int(sys.argv[1])
    data_to_sing = sys.argv[2]
    timestamp = get_current_timestamp()

    participating_parties = get_parties(threshold + 1)
    internal_urls = get_inter_comm_urls(threshold + 1, IS_DOCKER)
    outside_ports = get_ports(threshold + 1, 8000)

    signature = asyncio.run(get_signature(timestamp, participating_parties, internal_urls, outside_ports, data_to_sing))
    print(f"Signature: {signature.hex()}")
    print(f"Timestamp: {timestamp}")
