import asyncio
import os
from python.utils.endpoint_triggers import trigger_keygen_endpoint


def generate_keys(n, expected_err_code):
    results = asyncio.run(trigger_keygen_endpoint(n))

    for result in results:
        assert result[0] == expected_err_code


def remove_keys(n):
    for i in range(1, n + 1):
        os.remove(f"target/release/local-share{i}.json")


class TestKeyGen13:
    def test_keygen_no_keys(self):
        """
        Tests that keys are correctly generated on a 3 newly set up machines.
        """
        number_of_parties = 3
        expected_err_code = 200
        generate_keys(number_of_parties, expected_err_code)
        # remove_keys(number_of_parties)

    def test_keygen_keys_already_present(self):
        """
        Tests that once the keys were generated on the machines, it is not
        possible to regenerate new (and overwrite old ones) using the key
        generation endpoint.
        """
        number_of_parties = 3
        expected_err_code = 403

        # generate_keys(number_of_parties, 200)
        generate_keys(number_of_parties, expected_err_code)
        # remove_keys(number_of_parties)


class TestKeyGen24:

    def test_keygen_no_keys(self):
        """
        Tests that keys are correctly generated on a 4 newly set up machines.
        """
        number_of_parties = 4
        expected_err_code = 200

        generate_keys(number_of_parties, expected_err_code)
        # remove_keys(number_of_parties)

    def test_keygen_keys_already_present(self):
        """
        Tests that once the keys were generated on the machines, it is not
        possible to regenerate new (and overwrite old ones) using the key
        generation endpoint.
        """
        number_of_parties = 4
        expected_err_code = 403

        # generate_keys(number_of_parties, 200)
        generate_keys(number_of_parties, expected_err_code)
        # remove_keys(number_of_parties)


class TestKeyGen12:
    def test_keygen_no_keys(self):
        number_of_parties = 2
        expected_err_code = 200

        generate_keys(number_of_parties, expected_err_code)


class TestKeyGen36:
    def test_keygen_no_keys(self):
        number_of_parties = 6
        expected_err_code = 200

        generate_keys(number_of_parties, expected_err_code)
