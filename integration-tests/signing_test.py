import asyncio
from setup_for_tests import *
from common import get_current_timestamp
from endpoint_triggers import trigger_sign_endpoint


DATA_TO_SIGN = "0ab6fd240a2d8673464e57c36dac68c89f1313b5280590ab512d2fcfa7fbe1c2"


def test_signing_on_all_party_combinations():
    """
    Verifies that all signing combinations, namely
    [1,2], [1,3], and [2,3] work.
    """
    timestamp = get_current_timestamp()

    server1_res, server2_res = asyncio.run(
        trigger_sign_endpoint(
            [1, 2],
            [URL0, URL1],
            [SERVER_PORT0, SERVER_PORT1],
            timestamp,
            DATA_TO_SIGN
        )
    )
    assert server1_res == 200
    assert server2_res == 200

    server1_res, server2_res = asyncio.run(
        trigger_sign_endpoint(
            [1, 3],
            [URL0, URL2],
            [SERVER_PORT0, SERVER_PORT2],
            timestamp,
            DATA_TO_SIGN
        )
    )
    assert server1_res == 200
    assert server2_res == 200

    server1_res, server2_res = asyncio.run(
        trigger_sign_endpoint(
            [2, 3],
            [URL1, URL2],
            [SERVER_PORT1, SERVER_PORT2],
            timestamp,
            DATA_TO_SIGN
        )
    )
    assert server1_res == 200
    assert server2_res == 200
