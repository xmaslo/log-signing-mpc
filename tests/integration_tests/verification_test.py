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
        internal_urls = get_inter_comm_urls(3, IS_DOCKER)
        outside_ports = get_ports(3, 8000)

        signature = asyncio.run(
            get_signature(timestamp,
                          [1, 2],
                          [internal_urls[0], internal_urls[1]],
                          [outside_ports[0], outside_ports[1]]))

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[1]}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[2]}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == 200
        assert response2[0] == 200
        assert response3[0] == 200


class TestVerify24:
    def test_verify_signature_on_all_parties(self):
        timestamp = get_current_timestamp()
        internal_urls = get_inter_comm_urls(4, IS_DOCKER)
        outside_ports = get_ports(4, 8000)

        signature = asyncio.run(
            get_signature(timestamp,
                          [1, 3, 4],
                          [internal_urls[0], internal_urls[2], internal_urls[3]],
                          [outside_ports[0], outside_ports[2], outside_ports[3]]))

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[1]}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[2]}", DATA_TO_SIGN, signature, timestamp))
        response4 = asyncio.run(
            trigger_verify_endpoint(BASE_URL + f":{outside_ports[3]}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == 200
        assert response2[0] == 200
        assert response3[0] == 200
        assert response4[0] == 200
