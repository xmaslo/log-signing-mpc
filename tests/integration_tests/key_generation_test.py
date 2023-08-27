import asyncio
from tests.endpoint_triggers import trigger_keygen_endpoint


def test_keygen_no_keys():
    """
    Tests that keys are correctly generated on a newly set up machines.
    """
    server1_res, server2_res, server3_res = asyncio.run(trigger_keygen_endpoint())

    assert server1_res[0] == 200
    assert server2_res[0] == 200
    assert server3_res[0] == 200


def test_keygen_keys_already_present():
    """
    Tests that once the keys were generated on the machines, it is not
    possible to regenerate new (and overwrite old ones) using the key
    generation endpoint.
    """
    server1_res, server2_res, server3_res = asyncio.run(trigger_keygen_endpoint())

    assert server1_res[0] == 403
    assert server2_res[0] == 403
    assert server3_res[0] == 403
