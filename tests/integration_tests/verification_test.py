import asyncio
from common.setup_for_tests import *
from integration_tests.signing_test import sign_data
from common.common import get_current_timestamp
from common.endpoint_triggers import trigger_verify_endpoint

DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


async def get_signature(timestamp, parties, urls, ports):
    responses = await \
        sign_data(
            parties,
            urls,
            ports,
            timestamp,
            DATA_TO_SIGN,
            1
        )

    if responses[0][0] == 200:
        return responses[0][1]

    print("Unable to obtain signature")
    return None


class TestVerify13:
    def test_verify_signature_on_all_parties(self):
        timestamp = get_current_timestamp()
        signature = asyncio.run(
            get_signature(timestamp,
                          [1, 2],
                          [URL1, URL2],
                          [SERVER_PORT1, SERVER_PORT2]))

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT1}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT2}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT3}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == 200
        assert response2[0] == 200
        assert response3[0] == 200


class TestVerify24:
    def test_verify_signature_on_all_parties(self):
        timestamp = get_current_timestamp()
        signature = asyncio.run(
            get_signature(timestamp,
                          [1, 3, 4],
                          [URL1, URL3, URL4],
                          [SERVER_PORT1, SERVER_PORT3, SERVER_PORT4]))

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT1}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT2}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT3}", DATA_TO_SIGN, signature, timestamp))
        response4 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{SERVER_PORT4}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == 200
        assert response2[0] == 200
        assert response3[0] == 200
        assert response4[0] == 200
