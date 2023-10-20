import asyncio
from common.setup_for_tests import *
from integration_tests.signing_test import sign_data
from common.common import get_current_timestamp
from common.endpoint_triggers import trigger_verify_endpoint


DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


async def get_signature(timestamp):
    server1_res, _ = await \
        sign_data(
            [1, 2],
            [URL1, URL2],
            [SERVER_PORT1, SERVER_PORT2],
            timestamp,
            DATA_TO_SIGN,
            1
        )

    if server1_res[0] == 200:
        return server1_res[1]

    print("Unable to obtain signature")
    return None


class TestVerify13:
    def test_verify_signature_on_all_parties(self):
        """
        Tests that we can verify signature on all servers.
        """
        timestamp = get_current_timestamp()
        signature = asyncio.run(get_signature(timestamp))

        response1 = asyncio.run(trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT1}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT2}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT3}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == 200
        assert response2[0] == 200
        assert response3[0] == 200
