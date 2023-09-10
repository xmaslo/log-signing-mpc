from common.common import get_current_timestamp
from common.setup_for_tests import *
from common.endpoint_triggers import trigger_sign_endpoint
import asyncio
import aiohttp
import fileinput
import time


LOG_FILE_NAME = 'tests/log_files/nginx_json_logs.txt'


async def send_n_logs_for_signature_in_order(number_of_logs, file_with_logs):
    start_time = time.time()

    async with aiohttp.ClientSession() as session:
        counter = 1
        for line in fileinput.input([file_with_logs]):
            if counter == number_of_logs:
                break
            counter += 1

            timestamp = get_current_timestamp()
            server1_res, server2_res = await trigger_sign_endpoint(session,
                                                                   [1, 2],
                                                                   [URL0, URL1],
                                                                   [SERVER_PORT0, SERVER_PORT1],
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


def test_signing_10_logs_in_order():
    asyncio.run(send_n_logs_for_signature_in_order(10, LOG_FILE_NAME))


def send_n_logs_for_signature_in_parallel(number_of_logs, file_with_logs):
    pass


def test_signing_10_logs_in_parallel():
    pass