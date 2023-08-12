import asyncio
import aiohttp
from setup_for_tests import *
from common import send_post_request


async def trigger_keygen_endpoint(keys_already_generated):
    payload1 = URL1 + "," + URL2
    payload2 = URL2 + "," + URL0
    payload3 = URL1 + "," + URL0

    async with aiohttp.ClientSession() as session:
        tasks = [
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT0}/key_gen/1", payload1),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT1}/key_gen/1", payload2),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT2}/key_gen/1", payload3),
        ]

        server1_res, server2_res, server3_res = await asyncio.gather(*tasks)

    if not keys_already_generated:
        assert server1_res == 200
        assert server2_res == 200
        assert server3_res == 200
    else:
        assert server1_res == 403
        assert server2_res == 403
        assert server3_res == 403


def test_keygen_no_keys():
    """
    Tests that keys are correctly generated on a newly set up machines.
    """
    asyncio.run(trigger_keygen_endpoint(False))


def test_keygen_keys_already_present():
    """
    Tests that once the keys were generated on the machines, it is not
    possible to regenerate new (and overwrite old ones) using the key
    generation endpoint.
    """
    asyncio.run(trigger_keygen_endpoint(True))
