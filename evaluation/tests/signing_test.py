import asyncio
from evaluation.setup import *
from evaluation.utils.common import get_current_timestamp
from evaluation.utils.signatures import sign_data, run_parallel_signatures


DATA_TO_SIGN1 = "{some,arbitrary,data,to,sign}"
DATA_TO_SIGN2 = "{another,arbitrary,data}"


class TestSigning13:
    def test_signing_on_all_party_combinations(self):
        timestamp = get_current_timestamp()

        internal_urls = get_inter_comm_urls(3, IS_DOCKER)
        outside_ports = get_ports(3, 8000)

        responses = asyncio.run(
            sign_data(
                [1, 2],
                [internal_urls[0], internal_urls[1]],
                [outside_ports[0], outside_ports[1]],
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
                [internal_urls[0], internal_urls[2]],
                [outside_ports[0], outside_ports[2]],
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
                [internal_urls[1], internal_urls[2]],
                [outside_ports[1], outside_ports[2]],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200

    def test_parallel_signatures(self):
        number_of_parallel_signatures = 5

        internal_urls = get_inter_comm_urls(3, IS_DOCKER)
        outside_ports = get_ports(3, 8000)

        responses = run_parallel_signatures(number_of_parallel_signatures,
                                            [DATA_TO_SIGN1] * number_of_parallel_signatures,
                                            [2, 3],
                                            [internal_urls[1], internal_urls[2]],
                                            [outside_ports[1], outside_ports[2]])

        for i in range(0, number_of_parallel_signatures, 2):
            assert responses[i][0][0] == 200
            assert responses[i][1][0] == 200


class TestSigning24:
    def test_signing_on_all_party_combinations(self):
        timestamp = get_current_timestamp()

        internal_urls = get_inter_comm_urls(4, IS_DOCKER)
        outside_ports = get_ports(4, 8000)

        responses = asyncio.run(
            sign_data(
                [1, 2, 3],
                [internal_urls[0], internal_urls[1], internal_urls[2]],
                [outside_ports[0], outside_ports[1], outside_ports[2]],
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
                [internal_urls[1], internal_urls[2], internal_urls[3]],
                [outside_ports[1], outside_ports[2], outside_ports[3]],
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
                [internal_urls[0], internal_urls[1], internal_urls[3]],
                [outside_ports[0], outside_ports[1], outside_ports[3]],
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
                [internal_urls[0], internal_urls[2], internal_urls[3]],
                [outside_ports[0], outside_ports[2], outside_ports[3]],
                timestamp,
                DATA_TO_SIGN1,
                1
            )
        )
        assert responses[0][0] == 200
        assert responses[1][0] == 200
        assert responses[2][0] == 200

    def test_parallel_signatures(self):
        number_of_parallel_signatures = 5

        internal_urls = get_inter_comm_urls(4, IS_DOCKER)
        outside_ports = get_ports(4, 8000)

        responses = run_parallel_signatures(number_of_parallel_signatures,
                                            [DATA_TO_SIGN1] * number_of_parallel_signatures,
                                            [2, 3, 4],
                                            [internal_urls[1], internal_urls[2], internal_urls[3]],
                                            [outside_ports[1], outside_ports[2], outside_ports[3]])

        for i in range(0, number_of_parallel_signatures):
            assert responses[i][0][0] == 200
            assert responses[i][1][0] == 200
            assert responses[i][2][0] == 200
