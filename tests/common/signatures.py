import aiohttp
from common.endpoint_triggers import trigger_sign_endpoint, trigger_sign_endpoint_in_multiple_rooms


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
