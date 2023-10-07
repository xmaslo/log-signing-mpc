import asyncio
from common.endpoint_triggers import trigger_keygen_endpoint


def test_keygen_no_keys():
    """
    Tests that keys are correctly generated on a newly set up machines.
    """
    results = asyncio.run(trigger_keygen_endpoint(3))

    for result in results:
        assert result[0] == 200


def test_keygen_keys_already_present():
    """
    Tests that once the keys were generated on the machines, it is not
    possible to regenerate new (and overwrite old ones) using the key
    generation endpoint.
    """
    results = asyncio.run(trigger_keygen_endpoint(3))

    for result in results:
        assert result[0] == 403
