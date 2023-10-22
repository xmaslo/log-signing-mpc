from common.common import get_current_timestamp
from common.setup_for_tests import *
from common.endpoint_triggers import trigger_sign_endpoint
from common.signatures import run_parallel_signatures
import asyncio
import aiohttp
import fileinput
import time


LOG_FILE_NAME = 'tests/log_files/nginx_json_logs.txt'


async def send_n_logs_for_signature_in_order(number_of_logs, file_with_logs, parties, urls, ports):
    start_time = time.time()

    async with aiohttp.ClientSession() as session:
        counter = 0
        for line in fileinput.input([file_with_logs]):
            if counter == number_of_logs:
                break
            counter += 1

            timestamp = get_current_timestamp()
            server1_res, server2_res = await trigger_sign_endpoint(session,
                                                                   parties,
                                                                   urls,
                                                                   ports,
                                                                   timestamp,
                                                                   line,
                                                                   1
                                                                   )

            assert server1_res[0] == 200
            assert server2_res[0] == 200

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
    for server1_res, server2_res in responses:
        assert server1_res[0] and server1_res[1]
        assert server2_res[0] and server2_res[1]

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"\nExecution time: {execution_time:.2f} seconds")
    print(f"Execution time per log: {number_of_logs/execution_time:.2f} log/sec")

    fileinput.close()


class TestPerformance13:
    def test_signing_10_logs_in_order(self):
        asyncio.run(send_n_logs_for_signature_in_order(10,
                                                       LOG_FILE_NAME,
                                                       [2, 3],
                                                       [URL2, URL3],
                                                       [SERVER_PORT2, SERVER_PORT3]))

    def test_signing_10_logs_in_parallel(self):
        send_n_logs_for_signature_in_parallel(10,
                                              LOG_FILE_NAME,
                                              [2, 3],
                                              [URL2, URL3],
                                              [SERVER_PORT2, SERVER_PORT3])
