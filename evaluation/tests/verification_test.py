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


def assert_all_parties(n, response_code, signature, timestamp):
    outside_ports = get_ports(n, 8000)

    for i in range(n):
        response = asyncio.run(
            trigger_verify_endpoint(BASE_URL_HTTP + f":{outside_ports[0]}", DATA_TO_SIGN, signature, timestamp))

        assert response[0] == response_code


class TestVerify13:
    def test_verify_signature_on_all_parties(self):
        signature, timestamp = compute_signature(2, DATA_TO_SIGN)

        assert_all_parties(3, 200, signature, timestamp)


class TestVerify24:
    def test_verify_signature_on_all_parties(self):
        signature, timestamp = compute_signature(3, DATA_TO_SIGN)

        assert_all_parties(4, 200, signature, timestamp)
