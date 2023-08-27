import asyncio
import aiohttp
from common.setup_for_tests import *
from common.common import send_post_request


async def trigger_keygen_endpoint():
    payload1 = URL1 + "," + URL2
    payload2 = URL2 + "," + URL0
    payload3 = URL1 + "," + URL0

    async with aiohttp.ClientSession() as session:
        tasks = [
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT0}/key_gen/1", payload1),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT1}/key_gen/1", payload2),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT2}/key_gen/1", payload3),
        ]

        return await asyncio.gather(*tasks)


async def trigger_sign_endpoint(session, participating_parties, urls, ports, timestamp, data):
    data = data.encode().hex()
    payload1 = f"{str(participating_parties[1])}," + f"{urls[1]}," + data + "," + timestamp
    payload2 = f"{str(participating_parties[0])}," + f"{urls[0]}," + data + "," + timestamp

    tasks = [
        send_post_request(session, f"{BASE_URL}:{ports[0]}/sign/2", payload1),
        send_post_request(session, f"{BASE_URL}:{ports[1]}/sign/2", payload2),
    ]

    return await asyncio.gather(*tasks)


async def trigger_verify_endpoint(url, data_to_sign, signature, timestamp):
    data_to_sign = data_to_sign.encode().hex()
    payload = f"{signature.hex()},{data_to_sign},{timestamp}"
    async with aiohttp.ClientSession() as session:
        return await send_post_request(session, f"{url}/verify", payload)
