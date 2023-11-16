import aiohttp
import asyncio
from python_utils.endpoint_triggers import trigger_sign_endpoint, trigger_sign_endpoint_in_multiple_rooms
from python_utils.common import get_current_timestamp


async def sign_data(participating_parties, urls, ports, timestamp, data, room):
    async with aiohttp.ClientSession() as session:
        responses = await trigger_sign_endpoint(session,
                                                participating_parties,
                                                urls,
                                                ports,
                                                timestamp,
                                                data,
                                                room
                                                )

        return responses


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


def run_parallel_signatures(number_of_parallel_signatures, data_to_sign, parties, urls, ports):
    assert number_of_parallel_signatures == len(data_to_sign)

    timestamp = get_current_timestamp()

    responses = asyncio.run(
        sign_data_in_parallel(
            parties,
            urls,
            ports,
            timestamp,
            data_to_sign,
            [i for i in range(1, number_of_parallel_signatures + 1)]
        )
    )

    grouped_responses = []
    for i in range(0, len(responses), len(parties)):
        responses_for_room = responses[i: i + len(parties)]
        grouped_responses.append(responses_for_room)

    return grouped_responses
