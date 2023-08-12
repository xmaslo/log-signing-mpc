import aiohttp


async def send_post_request(session, url, payload):
    timeout = aiohttp.ClientTimeout(total=10)
    async with session.post(url=url, data=payload, timeout=timeout) as response:
        return response.status

