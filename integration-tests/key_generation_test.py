import asyncio
import aiohttp


IS_DOCKER = False
BASE_URL = "http://localhost"
SERVER_PORT1 = "8000"
SERVER_PORT2 = "8001"
SERVER_PORT3 = "8002"
if IS_DOCKER:
    URL0 = "la1:3000"
    URL1 = "la2:3001"
    URL2 = "la3:3002"
else:
    URL0 = "127.0.0.1:3000"
    URL1 = "127.0.0.1:3001"
    URL2 = "127.0.0.1:3002"


async def send_post_request(session, url, payload):
    async with session.post(url=url, data=payload) as response:
        return response.status


async def main():
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

        assert server1_res == 200
        assert server2_res == 200
        assert server3_res == 200


def test_keygen():
    asyncio.run(main())
