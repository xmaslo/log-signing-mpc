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
    print(f"Execution time per log: {signature_count/execution_time}")
    return (execution_time, signature_count/execution_time)


def compute_average(n, threshold, signature_count):
    cumulated_time = 0
    cumulated_average = 0
    for _ in range(n):
        result = verify_bench(threshold, signature_count)
        
        cumulated_time += result[0]
        cumulated_average += result[1]
    
    print(f"\nExecution time: {cumulated_time/n:.2f} seconds")
    print(f"Execution time per log: {cumulated_average/n:.2f} log/sec")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python verification.py <threshold> <number_of_signatures_to_verify> <number_of_trials>")
        sys.exit(1)

    compute_average(int(sys.argv[3]), int(sys.argv[1]), int(sys.argv[2]))
