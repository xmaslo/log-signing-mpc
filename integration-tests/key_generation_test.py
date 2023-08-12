import asyncio
import aiohttp
from setup_for_tests import *


async def send_post_request(session, url, payload):
    async with session.post(url=url, data=payload) as response:
        return response.status


async def trigger_keygen_endpoint(keys_already_generated):
    payload1 = URL1 + "," + URL2
    payload2 = URL2 + "," + URL0
    payload3 = URL1 + "," + URL0

    # Create a session for making asynchronous requests
    async with aiohttp.ClientSession() as session:
        # Use asyncio.gather to concurrently execute the requests
        tasks = [
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT1}/key_gen/1", payload1),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT2}/key_gen/1", payload2),
            send_post_request(session, f"{BASE_URL}:{SERVER_PORT3}/key_gen/1", payload3),
        ]

        responses = await asyncio.gather(*tasks)

    # Now you can access the responses as needed
    server1_res, server2_res, server3_res = responses

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
