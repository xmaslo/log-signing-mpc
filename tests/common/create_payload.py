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


def get_payloads_layout(server_ports, server_ids, server_urls):
    """
    Given, get_payloads_layout(["8001","8002","8003"], [1,2,3], ["url1","url2","url3"]),
    it returns:
        {
            "8001": [(2, "url2"), (3, "url3")],
            "8002": [(1, "url1), (3, "url3)],
            "8003": [(1, "url1"), (2, "url2")]
        }
    """
    payloads = {}

    assert len(server_ports) == len(server_ids)
    assert len(server_ids) == len(server_urls)
    n = len(server_ports)

    for i in range(n):
        other_ids = []
        for j in range(n):
            if i != j:
                other_ids.append((server_ids[j], server_urls[j]))
        payloads[server_ports[i]] = other_ids

    return payloads
