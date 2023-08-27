from common.common import get_current_timestamp
from common.setup_for_tests import *
from common.endpoint_triggers import trigger_sign_endpoint
import asyncio
import aiohttp
import fileinput


LOG_FILE_NAME = 'performance_tests/nginx_json_logs.txt'


async def send_n_logs_for_signature(number_of_logs):
    async with aiohttp.ClientSession() as session:
        counter = 1
        for line in fileinput.input([LOG_FILE_NAME]):
            if counter == number_of_logs:
                break
            counter += 1

            timestamp = get_current_timestamp()
            server1_res, server2_res = await trigger_sign_endpoint(session,
                                                                   [1, 2],
                                                                   [URL0, URL1],
                                                                   [SERVER_PORT0, SERVER_PORT1],
                                                                   timestamp,
                                                                   line
                                                                   )

            assert server1_res[0] == 200
            assert server2_res[0] == 200


def test_sending_10_logs():
    asyncio.run(send_n_logs_for_signature(10))
