import aiohttp
import asyncio
from common.endpoint_triggers import trigger_sign_endpoint, trigger_sign_endpoint_in_multiple_rooms
from common.common import get_current_timestamp
from common.setup_for_tests import *


async def sign_data(participating_parties, urls, ports, timestamp, data, room):
    async with aiohttp.ClientSession() as session:
        server1_res, server2_res = await trigger_sign_endpoint(session,
                                                               participating_parties,
                                                               urls,
                                                               ports,
                                                               timestamp,
                                                               data,
                                                               room
                                                               )

        return server1_res, server2_res


async def sign_data_in_parallel(participating_parties, urls, ports, timestamp, data_list, rooms):
    async with aiohttp.ClientSession() as session:
        responses = await trigger_sign_endpoint_in_multiple_rooms(session,
                                                                  participating_parties,
                                                                  urls,
                                                                  ports,
                                                                  timestamp,
                                                                  data_list,
                                                                  rooms
                                                                  )
        return responses


def run_parallel_signatures(number_of_parallel_signatures, data_to_sign):
    assert number_of_parallel_signatures == len(data_to_sign)

    timestamp = get_current_timestamp()

    responses = asyncio.run(
        sign_data_in_parallel(
            [2, 3],
            [URL1, URL2],
            [SERVER_PORT1, SERVER_PORT2],
            timestamp,
            data_to_sign,
            [i for i in range(1, number_of_parallel_signatures + 1)]
        )
    )

    grouped_responses = []
    for i in range(0, len(responses), 2):
        grouped_responses.append((responses[i], responses[i + 1]))

    return grouped_responses
