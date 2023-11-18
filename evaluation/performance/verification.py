import asyncio
import sys
import time

from evaluation.setup import get_inter_comm_urls, IS_DOCKER, get_ports, BASE_URL_HTTP, get_parties
from evaluation.utils.common import get_current_timestamp
from evaluation.utils.endpoint_triggers import trigger_verify_endpoint
from evaluation.utils.signatures import get_signature


DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


def verify_bench(threshold, signature_count):
    timestamp = get_current_timestamp()
    internal_urls = get_inter_comm_urls(threshold + 1, IS_DOCKER)
    outside_ports = get_ports(threshold + 1, 8000)
    parties = get_parties(threshold + 1)

    signature = asyncio.run(
        get_signature(timestamp,
                      parties,
                      internal_urls,
                      outside_ports,
                      DATA_TO_SIGN))

    start_time = time.time()

    for i in range(signature_count):
        response = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))
        assert response[0] == 200

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"Execution time: {execution_time:.2f} seconds")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python verification.py <threshold> <number_of_signatures_to_verify>")
        sys.exit(1)

    verify_bench(int(sys.argv[1]), int(sys.argv[2]))
