from python.utils.keygen import generate_keys


class TestKeyGen13:
    def test_keygen_no_keys(self):
        """
        Tests that keys are correctly generated on a 3 newly set up machines.
        """
        number_of_parties = 3
        expected_err_code = 200
        generate_keys(number_of_parties, expected_err_code)

    def test_keygen_keys_already_present(self):
        """
        Tests that once the keys were generated on the machines, it is not
        possible to regenerate new (and overwrite old ones) using the key
        generation endpoint.
        """
        number_of_parties = 3
        expected_err_code = 403

        generate_keys(number_of_parties, expected_err_code)


class TestKeyGen24:

    def test_keygen_no_keys(self):
        """
        Tests that keys are correctly generated on a 4 newly set up machines.
        """
        number_of_parties = 4
        expected_err_code = 200

        generate_keys(number_of_parties, expected_err_code)

    def test_keygen_keys_already_present(self):
        """
        Tests that once the keys were generated on the machines, it is not
        possible to regenerate new (and overwrite old ones) using the key
        generation endpoint.
        """
        number_of_parties = 4
        expected_err_code = 403

        generate_keys(number_of_parties, expected_err_code)


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
