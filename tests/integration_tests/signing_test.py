import asyncio
from common.setup_for_tests import *
from common.common import get_current_timestamp
from common.signatures import sign_data, run_parallel_signatures


DATA_TO_SIGN1 = "{some,arbitrary,data,to,sign}"
DATA_TO_SIGN2 = "{another,arbitrary,data}"


class TestSigning13:
    def test_sign_data(self):
        timestamp = get_current_timestamp()

        responses = asyncio.run(
            sign_data(
                [1, 2],
                [URL1, URL2],
                [SERVER_PORT1, SERVER_PORT2],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200

    def test_signing_on_all_party_combinations(self):
        timestamp = get_current_timestamp()

        responses = asyncio.run(
            sign_data(
                [1, 2],
                [URL1, URL2],
                [SERVER_PORT1, SERVER_PORT2],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200

        responses = asyncio.run(
            sign_data(
                [1, 3],
                [URL1, URL3],
                [SERVER_PORT1, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200

        responses = asyncio.run(
            sign_data(
                [2, 3],
                [URL2, URL3],
                [SERVER_PORT2, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200

    def test_parallel_signatures(self):
        number_of_parallel_signatures = 2

        responses = run_parallel_signatures(number_of_parallel_signatures,
                                            [DATA_TO_SIGN1, DATA_TO_SIGN2],
                                            [2, 3],
                                            [URL2, URL3],
                                            [SERVER_PORT2, SERVER_PORT3])

        for i in range(0, number_of_parallel_signatures, 2):
            assert responses[i][0][0] == 200
            assert responses[i][1][0] == 200


class TestSigning24:
    def test_sign_data(self):
        timestamp = get_current_timestamp()

        responses = asyncio.run(
            sign_data(
                [1, 2, 3],
                [URL1, URL2, URL3],
                [SERVER_PORT1, SERVER_PORT2, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

    def test_signing_on_all_party_combinations(self):
        timestamp = get_current_timestamp()

        responses = asyncio.run(
            sign_data(
                [1, 2, 3],
                [URL1, URL2, URL3],
                [SERVER_PORT1, SERVER_PORT2, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

        responses = asyncio.run(
            sign_data(
                [2, 3, 4],
                [URL2, URL3, URL4],
                [SERVER_PORT2, SERVER_PORT3, SERVER_PORT4],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

        responses = asyncio.run(
            sign_data(
                [1, 2, 4],
                [URL1, URL2, URL4],
                [SERVER_PORT1, SERVER_PORT2, SERVER_PORT4],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

        responses = asyncio.run(
            sign_data(
                [1, 3, 4],
                [URL1, URL3, URL4],
                [SERVER_PORT1, SERVER_PORT3, SERVER_PORT4],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

    def test_parallel_signatures(self):
        number_of_parallel_signatures = 2

        responses = run_parallel_signatures(number_of_parallel_signatures,
                                            [DATA_TO_SIGN1, DATA_TO_SIGN2],
                                            [2, 3, 4],
                                            [URL2, URL3, URL4],
                                            [SERVER_PORT2, SERVER_PORT3, SERVER_PORT4])

        for i in range(0, number_of_parallel_signatures):
            assert responses[i][0][0] == 200
            assert responses[i][1][0] == 200
            assert responses[i][2][0] == 200
