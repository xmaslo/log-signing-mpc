import asyncio
import os
from common.endpoint_triggers import trigger_keygen_endpoint


def generate_keys(n, expected_err_code):
    results = asyncio.run(trigger_keygen_endpoint(3))

    for result in results:
        assert result[0] == expected_err_code


def remove_keys(n):
    for i in range(1, n + 1):
        os.remove(f"target/release/local-share{i}.json")


def test_keygen_no_keys_3():
    """
    Tests that keys are correctly generated on a newly set up machines.
    """
    number_of_parties = 3
    expected_err_code = 200

    generate_keys(number_of_parties, expected_err_code)
    remove_keys(number_of_parties)


def test_keygen_keys_already_present_3():
    """
    Tests that once the keys were generated on the machines, it is not
    possible to regenerate new (and overwrite old ones) using the key
    generation endpoint.
    """
    number_of_parties = 3

    generate_keys(number_of_parties, 200)
    generate_keys(number_of_parties, 403)
    remove_keys(number_of_parties)
