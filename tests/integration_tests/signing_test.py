import asyncio
from common.setup_for_tests import *
from common.common import get_current_timestamp
from common.signatures import sign_data, run_parallel_signatures


DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


class TestSigning13:
    def test_signing_on_all_party_combinations(self):
        """
        Verifies that all signing combinations, namely
        [1,2], [1,3], and [2,3] work.
        """
        timestamp = get_current_timestamp()

        responses = asyncio.run(
            sign_data(
                [1, 2],
                [URL1, URL2],
                [SERVER_PORT1, SERVER_PORT2],
                timestamp,
                DATA_TO_SIGN,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[0][0] == 200

        responses = asyncio.run(
            sign_data(
                [1, 3],
                [URL1, URL3],
                [SERVER_PORT1, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[0][0] == 200

        responses = asyncio.run(
            sign_data(
                [2, 3],
                [URL2, URL3],
                [SERVER_PORT2, SERVER_PORT3],
                timestamp,
                DATA_TO_SIGN,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[0][0] == 200

    def test_parallel_signatures(self):
        number_of_parallel_signatures = 2

        responses = run_parallel_signatures(number_of_parallel_signatures,
                                            [DATA_TO_SIGN for _ in range(number_of_parallel_signatures)])

        for i in range(0, number_of_parallel_signatures, 2):
            assert responses[i][0][0] == 200
            assert responses[i][1][0] == 200


class TestSigning14:
    def test_signing(self):
        timestamp = get_current_timestamp()
