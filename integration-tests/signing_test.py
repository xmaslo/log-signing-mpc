import asyncio
import aiohttp
import time
from setup_for_tests import *
from common import send_post_request


DATA_TO_SIGN = "0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2"


async def sign_data(participating_parties, urls, ports, data):
    current_time = str(int(time.time()))
    payload1 = f"{str(participating_parties[1])}," + f"{urls[1]}," + data + "," + current_time
    payload2 = f"{str(participating_parties[0])}," + f"{urls[0]}," + data + "," + current_time

    async with aiohttp.ClientSession() as session:
        tasks = [
            send_post_request(session, f"{BASE_URL}:{ports[0]}/sign/2", payload1),
            send_post_request(session, f"{BASE_URL}:{ports[1]}/sign/2", payload2),
        ]

        server1_res, server2_res = await asyncio.gather(*tasks)

    assert server1_res == 200
    assert server2_res == 200


def test_signing_on_all_party_combinations():
    """
    Verifies that all signing combinations, namely
    [1,2], [1,3], and [2,3] work.
    """
    asyncio.run(
        sign_data(
            [1, 2],
            [URL0, URL1],
            [SERVER_PORT1, SERVER_PORT2],
            DATA_TO_SIGN
        )
    )

    asyncio.run(
        sign_data(
            [1, 3],
            [URL0, URL2],
            [SERVER_PORT1, SERVER_PORT3],
            DATA_TO_SIGN
        )
    )

    asyncio.run(
        sign_data(
            [2, 3],
            [URL1, URL2],
            [SERVER_PORT2, SERVER_PORT3],
            DATA_TO_SIGN
        )
    )
