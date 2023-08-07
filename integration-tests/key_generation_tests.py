import asyncio
import aiohttp


BASE_URL = "http://localhost"
SERVER_PORT1 = "8000"
SERVER_PORT2 = "8001"
SERVER_PORT3 = "8002"


async def send_post_request(session, url, payload):
    async with session.post(url=url, data=payload) as response:
        return response.status


async def main():
    payload1 = "127.0.0.1:3001,127.0.0.1:3002"
    payload2 = "127.0.0.1:3002,127.0.0.1:3000"
    payload3 = "127.0.0.1:3001,127.0.0.1:3000"

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

        # TODO:
        # Use HTTP status code 200 for successful requests that retrieve or update a resource.
        # Use HTTP status code 201 for successful requests that create a new resource on the server.
        # Use HTTP status code 202 for requests that have been accepted for processing but the
        # processing has not yet been completed.
        print("Server 1 response:", server1_res)
        print("Server 2 response:", server2_res)
        print("Server 3 response:", server3_res)


if __name__ == "__main__":
    asyncio.run(main())
