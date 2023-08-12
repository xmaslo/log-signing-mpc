import aiohttp
import time


async def send_post_request(session, url, payload):
    """
    Sends post request.
    :param session: Aiohttp client session.
    :param url: Endpoint of a target.
    :param payload: Payload of the post request (plain data).
    :return: Response status code.
    """
    timeout = aiohttp.ClientTimeout(total=10)
    async with session.post(url=url, data=payload, timeout=timeout) as response:
        return response.status


def get_current_timestamp():
    """
    :return: Number of seconds from the beginning of the epoch.
    """
    return str(int(time.time()))

