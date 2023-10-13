import json


def create_sign_payload(server_ids, server_urls, data, timestamp):
    payload = {}
    assert len(server_ids) == len(server_urls)

    payload["participants"] = []
    for server_id, server_url in zip(server_ids, server_urls):
        payload["participants"].append({
            "server_id": server_id,
            "url": server_url
        })

    payload["data_to_sign"] = data

    payload["timestamp"] = timestamp

    return json.dumps(payload)

