import asyncio
import aiohttp
from common.setup_for_tests import *
from common.common import send_post_request
from common.create_payload import create_sign_payload


async def trigger_keygen_endpoint(n):
    urls = get_urls(n)
    payloads = get_keygen_payloads(n, IS_DOCKER)

    async with aiohttp.ClientSession() as session:
        tasks = []
        for i in range(n):
            tasks.append(send_post_request(session, f"{urls[i]}/key_gen/1", payloads[i]))

        return await asyncio.gather(*tasks)


async def trigger_sign_endpoint(session, participating_parties, urls, ports, timestamp, data, room):
    return await trigger_sign_endpoint_in_multiple_rooms(session,
                                                         participating_parties,
                                                         urls,
                                                         ports,
                                                         timestamp,
                                                         [data],
                                                         [room])


async def trigger_sign_endpoint_in_multiple_rooms(session,
                                                  participating_parties,
                                                  urls,
                                                  ports,
                                                  timestamp,
                                                  data_list,
                                                  rooms):
    assert len(data_list) == len(rooms)

    tasks = []

    for count, room in enumerate(rooms):
        data = data_list[count].encode().hex()
        payload1 = create_sign_payload([participating_parties[1]], [urls[1]], data, timestamp)
        payload2 = create_sign_payload([participating_parties[0]], [urls[0]], data, timestamp)

        tasks.append(send_post_request(session, f"{BASE_URL}:{ports[0]}/sign/{room}", payload1))
        tasks.append(send_post_request(session, f"{BASE_URL}:{ports[1]}/sign/{room}", payload2))

    return await asyncio.gather(*tasks)


async def trigger_verify_endpoint(url, data_to_sign, signature, timestamp):
    data_to_sign = data_to_sign.encode().hex()
    payload = f"{signature.hex()},{data_to_sign},{timestamp}"
    async with aiohttp.ClientSession() as session:
        return await send_post_request(session, f"{url}/verify", payload)
