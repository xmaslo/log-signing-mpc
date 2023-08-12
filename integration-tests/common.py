async def send_post_request(session, url, payload):
    async with session.post(url=url, data=payload) as response:
        return response.status

