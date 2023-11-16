import asyncio
import aiohttp
from common.setup_for_tests import *
from common.common import send_post_request
from common.create_payload import create_sign_payload, get_payloads_layout


def get_keygen_payloads(n, is_docker):
    payloads = []
    for i in range(n):
        payload = ""
        for j in range(n):
            if i != j:
                if is_docker:
                    payload += f"la{j + 1}:300{j + 1},"
                else:
                    payload += f"127.0.0.1:300{j + 1},"

        payloads.append(payload[:-1])  # remove trailing ','

    return payloads


async def trigger_keygen_endpoint(n):
    urls = get_endpoint_urls(n)
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

    payloads_layout = get_payloads_layout(ports, participating_parties, urls)

    for count, room in enumerate(rooms):
        data = data_list[count].encode().hex()

        payloads = {}
        for pl_key, pl_val in payloads_layout.items():
            payloads[pl_key] = create_sign_payload([x[0] for x in pl_val],
                                                   [x[1] for x in pl_val],
                                                   data,
                                                   timestamp)

        for pl_key, pl_val in payloads.items():
            tasks.append(send_post_request(session, f"{BASE_URL_HTTP}:{pl_key}/sign/{room}", pl_val))

    return await asyncio.gather(*tasks)


async def trigger_verify_endpoint(url, data_to_sign, signature, timestamp):
    data_to_sign = data_to_sign.encode().hex()
    payload = f"{signature.hex()},{data_to_sign},{timestamp}"
    async with aiohttp.ClientSession() as session:
        return await send_post_request(session, f"{url}/verify", payload)
