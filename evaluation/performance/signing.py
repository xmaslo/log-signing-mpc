import sys
from evaluation.utils.common import get_current_timestamp
from evaluation.setup import *
from evaluation.utils.endpoint_triggers import trigger_sign_endpoint
from evaluation.utils.signatures import run_parallel_signatures
import asyncio
import aiohttp
import fileinput
import time


async def send_n_logs_for_signature_in_order(number_of_logs, file_with_logs, parties, urls, ports):
    start_time = time.time()

    async with aiohttp.ClientSession() as session:
        counter = 0
        for line in fileinput.input([file_with_logs]):
            if counter == number_of_logs:
                break
            counter += 1

            timestamp = get_current_timestamp()
            responses = await trigger_sign_endpoint(session,
                                                    parties,
                                                    urls,
                                                    ports,
                                                    timestamp,
                                                    line,
                                                    1
                                                    )

            assert responses[0][0] == 200
            assert responses[0][0] == 200

        end_time = time.time()
        execution_time = end_time - start_time
        print(f"\nExecution time: {execution_time:.2f} seconds")
        print(f"Execution time per log: {number_of_logs/execution_time:.2f} log/sec")

    fileinput.close()


def send_n_logs_for_signature_in_parallel(number_of_logs, file_with_logs, participants, urls, ports):
    start_time = time.time()

    counter = 0
    logs = []
    for line in fileinput.input([file_with_logs]):
        if counter == number_of_logs:
            break
        counter += 1
        logs.append(line)

    responses = run_parallel_signatures(number_of_logs, logs, participants, urls, ports)
    for room_responses in responses:
        for rp in room_responses:
            assert rp[0] and rp[1]

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"\nExecution time: {execution_time:.2f} seconds")
    print(f"Execution time per log: {number_of_logs/execution_time:.2f} log/sec")

    fileinput.close()


def benchmark(parallel_n, log_file_name, parties_n, internals_n, ports_n, t_type):
    if t_type == "order":
        asyncio.run(send_n_logs_for_signature_in_order(parallel_n,
                                                       log_file_name,
                                                       parties_n,
                                                       internals_n,
                                                       ports_n))
    elif t_type == "parallel":
        send_n_logs_for_signature_in_parallel(parallel_n,
                                              log_file_name,
                                              parties_n,
                                              internals_n,
                                              ports_n)
    else:
        print(f"Unknown test type: {t_type}")


LOG_FILE_NAME = 'log_files/nginx_json_logs.txt'


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python signing.py <threshold> <test_type> <log_count>")
        sys.exit(1)

    threshold = int(sys.argv[1])
    test_type = sys.argv[2]
    parallel_count = int(sys.argv[3])

    participating_parties = get_parties(threshold + 1)
    internal_urls = get_inter_comm_urls(threshold + 1, IS_DOCKER)
    outside_ports = get_ports(threshold + 1, 8000)

    benchmark(parallel_count,
              LOG_FILE_NAME,
              participating_parties,
              internal_urls,
              outside_ports,
              test_type)
