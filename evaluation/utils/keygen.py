import asyncio
from evaluation.utils.endpoint_triggers import trigger_keygen_endpoint


def generate_keys(n, expected_err_code):
    results = asyncio.run(trigger_keygen_endpoint(n))

    for result in results:
        assert result[0] == expected_err_code
