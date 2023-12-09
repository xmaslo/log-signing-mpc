import asyncio
from evaluation.setup import *
from evaluation.utils.common import get_current_timestamp
from evaluation.utils.endpoint_triggers import trigger_verify_endpoint
from evaluation.utils.signatures import get_signature


DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


def compute_signature(n, data_to_sign):
    timestamp = get_current_timestamp()
    internal_urls = get_inter_comm_urls(n, IS_DOCKER)
    outside_ports = get_ports(n, 8000)

    signature = asyncio.run(
        get_signature(timestamp,
                      list(range(1, n+1)),
                      internal_urls,
                      outside_ports,
                      data_to_sign))

    return signature, timestamp


class TestVerify13:
    def test_verify_signature_on_all_parties(self):
        response_code = 200

        signature, timestamp = compute_signature(2, DATA_TO_SIGN)
        outside_ports = get_ports(3, 8000)

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[1]}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[2]}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == response_code
        assert response2[0] == response_code
        assert response3[0] == response_code


class TestVerify24:
    def test_verify_signature_on_all_parties(self):
        response_code = 200

        signature, timestamp = compute_signature(3, DATA_TO_SIGN)
        outside_ports = get_ports(4, 8000)

        response1 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))
        response2 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[1]}", DATA_TO_SIGN, signature, timestamp))
        response3 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[2]}", DATA_TO_SIGN, signature, timestamp))
        response4 = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[3]}", DATA_TO_SIGN, signature, timestamp))

        assert response1[0] == response_code
        assert response2[0] == response_code
        assert response3[0] == response_code
        assert response4[0] == response_code
