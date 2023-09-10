import asyncio
from common.setup_for_tests import *
from common.common import get_current_timestamp
from common.signatures import sign_data, run_parallel_signatures


DATA_TO_SIGN = "{some,arbitrary,data,to,sign}"


def test_signing_on_all_party_combinations():
    """
    Verifies that all signing combinations, namely
    [1,2], [1,3], and [2,3] work.
    """
    timestamp = get_current_timestamp()

    server1_res, server2_res = asyncio.run(
        sign_data(
            [1, 2],
            [URL0, URL1],
            [SERVER_PORT0, SERVER_PORT1],
            timestamp,
            DATA_TO_SIGN,
            1
        )
    )
    assert server1_res[0] == 200
    assert server2_res[0] == 200

    server1_res, server2_res = asyncio.run(
        sign_data(
            [1, 3],
            [URL0, URL2],
            [SERVER_PORT0, SERVER_PORT2],
            timestamp,
            DATA_TO_SIGN,
            1
        )
    )
    assert server1_res[0] == 200
    assert server2_res[0] == 200

    server1_res, server2_res = asyncio.run(
        sign_data(
            [2, 3],
            [URL1, URL2],
            [SERVER_PORT1, SERVER_PORT2],
            timestamp,
            DATA_TO_SIGN,
            1
        )
    )
    assert server1_res[0] == 200
    assert server2_res[0] == 200


def test_parallel_signatures():
    number_of_parallel_signatures = 2

    responses = run_parallel_signatures(number_of_parallel_signatures,
                                        [DATA_TO_SIGN for _ in range(number_of_parallel_signatures)])

    for i in range(0, number_of_parallel_signatures, 2):
        assert responses[i][0][0] == 200
        assert responses[i][1][0] == 200
