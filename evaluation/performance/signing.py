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

    return (execution_time, number_of_logs/execution_time)


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

    return (execution_time, number_of_logs/execution_time)


def benchmark(parallel_n, log_file_name, parties_n, internals_n, ports_n, t_type):
    if t_type == "order":
        return asyncio.run(send_n_logs_for_signature_in_order(parallel_n,
                                                              log_file_name,
                                                              parties_n,
                                                              internals_n,
                                                              ports_n))
    elif t_type == "parallel":
        return send_n_logs_for_signature_in_parallel(parallel_n,
                                                     log_file_name,
                                                     parties_n,
                                                     internals_n,
                                                     ports_n)
    else:
        print(f"Unknown test type: {t_type}")
        return None


def compute_average(n, p_count, file_name, parties, urls, ports, tt):
    # the first signature should be ignored
    asyncio.run(send_n_logs_for_signature_in_order(1,
                                                   file_name,
                                                   parties,
                                                   urls,
                                                   ports))

    cumulated_time = 0
    cumulated_average = 0
    for _ in range(n):
        result = benchmark(p_count,
                           file_name,
                           parties,
                           urls,
                           ports,
                           tt)
        
        cumulated_time += result[0]
        cumulated_average += result[1]
    
    print(f"\nExecution time: {cumulated_time/n:.2f} seconds")
    print(f"Execution time per log: {cumulated_average/n:.2f} log/sec")


LOG_FILE_NAME = 'log_files/nginx_json_logs.txt'


if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: python signing.py <threshold> <test_type> <log_count> <number_of_trials>")
        sys.exit(1)

    threshold = int(sys.argv[1])
    test_type = sys.argv[2]
    parallel_count = int(sys.argv[3])
    number_of_trials = int(sys.argv[4])

    participating_parties = get_parties(threshold + 1)
    internal_urls = get_inter_comm_urls(threshold + 1, IS_DOCKER)
    outside_ports = get_ports(threshold + 1, 8000)

    compute_average(number_of_trials,
                    parallel_count,
                    LOG_FILE_NAME,
                    participating_parties,
                    internal_urls,
                    outside_ports,
                    test_type)
