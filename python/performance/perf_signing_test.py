from python.utils.common import get_current_timestamp
from python.utils.setup_for_tests import *
from python.utils.endpoint_triggers import trigger_sign_endpoint
from python.utils.signatures import run_parallel_signatures
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


class TestPerformance13:
    def test_signing_10_logs_in_order(self):
        internal_urls = get_inter_comm_urls(3, IS_DOCKER)
        outside_ports = get_ports(3, 8000)

        asyncio.run(send_n_logs_for_signature_in_order(10,
                                                       LOG_FILE_NAME,
                                                       [2, 3],
                                                       [internal_urls[1], internal_urls[2]],
                                                       [outside_ports[1], outside_ports[2]]))

    def test_signing_10_logs_in_parallel(self):
        internal_urls = get_inter_comm_urls(3, IS_DOCKER)
        outside_ports = get_ports(3, 8000)

        send_n_logs_for_signature_in_parallel(10,
                                              LOG_FILE_NAME,
                                              [2, 3],
                                              [internal_urls[1], internal_urls[2]],
                                              [outside_ports[1], outside_ports[2]])


class TestPerformance24:
    def test_signing_10_logs_in_order(self):
        internal_urls = get_inter_comm_urls(4, IS_DOCKER)
        outside_ports = get_ports(4, 8000)

        asyncio.run(send_n_logs_for_signature_in_order(10,
                                                       LOG_FILE_NAME,
                                                       [2, 3, 4],
                                                       [internal_urls[1], internal_urls[2], internal_urls[3]],
                                                       [outside_ports[1], outside_ports[2], outside_ports[3]]))

    def test_signing_10_logs_in_parallel(self):
        internal_urls = get_inter_comm_urls(4, IS_DOCKER)
        outside_ports = get_ports(4, 8000)

        send_n_logs_for_signature_in_parallel(10,
                                              LOG_FILE_NAME,
                                              [2, 3, 4],
                                              [internal_urls[1], internal_urls[2], internal_urls[3]],
                                              [outside_ports[1], outside_ports[2], outside_ports[3]])


class TestPerformance12:
    def test_signing_10_logs_in_parallel(self):
        internal_urls = get_inter_comm_urls(2, IS_DOCKER)
        outside_ports = get_ports(2, 8000)

        send_n_logs_for_signature_in_parallel(10,
                                              LOG_FILE_NAME,
                                              [1, 2],
                                              [internal_urls[0], internal_urls[1]],
                                              [outside_ports[0], outside_ports[1]])


class TestPerformance36:
    def test_signing_10_logs_in_parallel(self):
        internal_urls = get_inter_comm_urls(6, IS_DOCKER)
        outside_ports = get_ports(6, 8000)

        send_n_logs_for_signature_in_parallel(10,
                                              LOG_FILE_NAME,
                                              [1, 2, 3, 4],
                                              [internal_urls[0], internal_urls[1], internal_urls[2], internal_urls[3]],
                                              [outside_ports[0], outside_ports[1], outside_ports[2], outside_ports[3]])
